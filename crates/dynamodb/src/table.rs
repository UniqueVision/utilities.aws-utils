use crate::error::{Error, from_aws_sdk_error};
use aws_sdk_dynamodb::{
    Client,
    operation::{
        create_table::CreateTableOutput, delete_table::DeleteTableOutput,
        describe_table::DescribeTableOutput, update_table::UpdateTableOutput,
    },
    types::{AttributeDefinition, BillingMode, KeySchemaElement, KeyType, ProvisionedThroughput},
};
use aws_smithy_types_convert::stream::PaginationStreamExt;
use futures_util::{Stream, TryStreamExt};

pub enum TableType {
    OnDemand,
    Provisioned(i64, i64),
}

pub async fn create_table(
    client: &Client,
    table_name: impl Into<String>,
    hash_key_name: impl Into<String>,
    sort_key_name: Option<impl Into<String>>,
    table_type: TableType,
    attribute_definitions: Vec<AttributeDefinition>,
    global_secondary_indexes: Option<Vec<aws_sdk_dynamodb::types::GlobalSecondaryIndex>>,
) -> Result<CreateTableOutput, Error> {
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

    let table_builder = client
        .create_table()
        .table_name(table_name)
        .set_key_schema(Some(kss))
        .set_global_secondary_indexes(global_secondary_indexes)
        .set_attribute_definitions(Some(attribute_definitions));

    match table_type {
        TableType::OnDemand => table_builder
            .billing_mode(BillingMode::PayPerRequest)
            .send()
            .await
            .map_err(from_aws_sdk_error),
        TableType::Provisioned(read_capacity, write_capacity) => {
            let pt = ProvisionedThroughput::builder()
                .read_capacity_units(read_capacity)
                .write_capacity_units(write_capacity)
                .build()?;
            table_builder
                .provisioned_throughput(pt)
                .send()
                .await
                .map_err(from_aws_sdk_error)
        }
    }
}

pub async fn delete_table(
    client: &Client,
    table_name: impl Into<String>,
) -> Result<DeleteTableOutput, Error> {
    client
        .delete_table()
        .table_name(table_name)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub fn list_tables_stream(client: &Client) -> impl Stream<Item = Result<String, Error>> {
    client
        .list_tables()
        .into_paginator()
        .items()
        .send()
        .into_stream_03x()
        .map_err(from_aws_sdk_error)
}

pub async fn delete_tables(client: &Client) -> Result<(), Error> {
    let stream = list_tables_stream(client);
    futures_util::pin_mut!(stream);
    while let Some(table_name) = stream.try_next().await? {
        delete_table(client, table_name).await?;
    }
    Ok(())
}

pub async fn describe_table(
    client: &Client,
    table_name: impl Into<String>,
) -> Result<DescribeTableOutput, Error> {
    client
        .describe_table()
        .table_name(table_name)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn get_capacity(
    client: &Client,
    table_name: impl Into<String>,
) -> Result<(i64, i64), Error> {
    let res = describe_table(client, table_name).await?;
    let Some(table) = res.table() else {
        return Err(Error::NotFound);
    };
    let Some(th) = table.provisioned_throughput() else {
        return Err(Error::ValidationError(
            "Table does not have provisioned throughput".to_string(),
        ));
    };
    Ok((
        th.read_capacity_units().unwrap_or_default(),
        th.write_capacity_units().unwrap_or_default(),
    ))
}

pub async fn set_capacity(
    client: &Client,
    table_name: &str,
    read_count: i64,
    write_count: i64,
) -> Result<UpdateTableOutput, Error> {
    let pt = ProvisionedThroughput::builder()
        .read_capacity_units(read_count)
        .write_capacity_units(write_count)
        .build()?;

    client
        .update_table()
        .table_name(table_name)
        .provisioned_throughput(pt)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}
