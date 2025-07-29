use std::time::Duration;

use aws_sdk_dynamodb::{
    Client,
    types::{
        AttributeDefinition, BillingMode, CsvOptions, ImportStatus, InputFormat,
        InputFormatOptions, KeySchemaElement, KeyType, ProvisionedThroughput, S3BucketSource,
        TableCreationParameters,
    },
};
use tokio::time::sleep;

use crate::{
    error::{Error, from_aws_sdk_error},
    table::TableType,
};

#[allow(clippy::too_many_arguments)]
pub async fn import_table(
    client: &Client,
    bucket_name: impl Into<String>,
    key_prefix: impl Into<String>,
    delimiter: Option<impl Into<String>>,
    header_list: Option<Vec<String>>,
    table_name: impl Into<String>,
    hash_key_name: impl Into<String>,
    sort_key_name: Option<impl Into<String>>,
    attribute_definitions: Vec<AttributeDefinition>,
    table_type: TableType,
) -> Result<(), Error> {
    let s3_bucket_source = S3BucketSource::builder()
        .s3_bucket(bucket_name)
        .s3_key_prefix(key_prefix)
        .build()?;

    let ks = KeySchemaElement::builder()
        .attribute_name(hash_key_name)
        .key_type(KeyType::Hash)
        .build()?;

    let kss = if let Some(sort_key_name) = sort_key_name {
        let sort_key = KeySchemaElement::builder()
            .attribute_name(sort_key_name)
            .key_type(KeyType::Range)
            .build()?;
        vec![ks, sort_key]
    } else {
        vec![ks]
    };

    let mut table_creation_parameters = TableCreationParameters::builder()
        .table_name(table_name)
        .set_key_schema(Some(kss))
        .set_attribute_definitions(Some(attribute_definitions));

    match table_type {
        TableType::OnDemand => {
            table_creation_parameters =
                table_creation_parameters.billing_mode(BillingMode::PayPerRequest)
        }
        TableType::Provisioned(read_capacity, write_capacity) => {
            let pt = ProvisionedThroughput::builder()
                .read_capacity_units(read_capacity)
                .write_capacity_units(write_capacity)
                .build()?;
            table_creation_parameters = table_creation_parameters.provisioned_throughput(pt);
        }
    }
    let table_creation_parameters = table_creation_parameters.build()?;

    let csv_options = CsvOptions::builder()
        .set_delimiter(delimiter.map(Into::into))
        .set_header_list(header_list)
        .build();

    let ifo = InputFormatOptions::builder()
        .set_csv(Some(csv_options))
        .build();

    let import_arn = client
        .import_table()
        .s3_bucket_source(s3_bucket_source)
        .input_format(InputFormat::Csv)
        .set_input_format_options(Some(ifo))
        .table_creation_parameters(table_creation_parameters)
        .send()
        .await
        .map_err(from_aws_sdk_error)?
        .import_table_description
        .ok_or(Error::Invalid("failed to get import_arn".to_string()))?
        .import_arn
        .ok_or(Error::Invalid("failed to get import_arn".to_string()))?;

    let mut count = 0;
    loop {
        let status = client
            .describe_import()
            .import_arn(import_arn.clone())
            .send()
            .await
            .map_err(from_aws_sdk_error)?
            .import_table_description
            .ok_or(Error::Invalid("failed to get status".to_string()))?
            .import_status
            .ok_or(Error::Invalid("failed to get status".to_string()))?;

        match status {
            ImportStatus::InProgress => {}
            ImportStatus::Completed => break,
            _ => {
                return Err(Error::Invalid("import_table failed".to_string()));
            }
        }

        count += 1;
        if count > 60 {
            return Err(Error::Invalid("import_table timeout".to_string()));
        }
        sleep(Duration::from_secs(10)).await;
    }
    Ok(())
}
