use aws_sdk_secretsmanager::{Client, operation::get_secret_value::GetSecretValueOutput};

use crate::error::{Error, from_aws_sdk_error};

pub async fn get_secret_value_raw(
    client: &Client,
    secret_id: Option<impl Into<String>>,
    version_id: Option<impl Into<String>>,
    version_stage: Option<impl Into<String>>,
) -> Result<GetSecretValueOutput, Error> {
    client
        .get_secret_value()
        .set_secret_id(secret_id.map(Into::into))
        .set_version_id(version_id.map(Into::into))
        .set_version_stage(version_stage.map(Into::into))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn get_secret_value(client: &Client, secret_id: &str) -> Result<String, Error> {
    let res = get_secret_value_raw(client, Some(secret_id), None::<String>, None::<String>).await?;
    res.secret_string()
        .ok_or_else(|| Error::NotFound)
        .map(|s| s.to_string())
}
