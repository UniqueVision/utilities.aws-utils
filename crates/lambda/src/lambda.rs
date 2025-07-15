use aws_sdk_lambda::{
    Client,
    operation::invoke::InvokeOutput,
    primitives::Blob,
    types::{InvocationType, LogType},
};

use crate::error::{Error, from_aws_sdk_error};

pub async fn invoke(
    client: &Client,
    function_name: Option<impl Into<String>>,
    client_context: Option<impl Into<String>>,
    invokation_type: Option<InvocationType>,
    log_type: Option<LogType>,
    payload: Option<impl Into<Blob>>,
    qualifier: Option<impl Into<String>>,
) -> Result<InvokeOutput, Error> {
    client
        .invoke()
        .set_client_context(client_context.map(|c| c.into()))
        .set_function_name(function_name.map(|f| f.into()))
        .set_invocation_type(invokation_type)
        .set_log_type(log_type)
        .set_payload(payload.map(|p| p.into()))
        .set_qualifier(qualifier.map(|q| q.into()))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}
