use aws_sdk_athena::{
    Client,
    operation::{
        get_query_execution::GetQueryExecutionOutput,
        start_query_execution::StartQueryExecutionOutput,
    },
    types::{QueryExecutionContext, ResultConfiguration},
};

use crate::error::{Error, from_aws_sdk_error};

pub async fn start_query_execution(
    client: &Client,
    query_string: Option<impl Into<String>>,
    query_execution_context: Option<QueryExecutionContext>,
    result_configuration: Option<ResultConfiguration>,
) -> Result<StartQueryExecutionOutput, Error> {
    client
        .start_query_execution()
        .set_query_string(query_string.map(Into::into))
        .set_query_execution_context(query_execution_context)
        .set_result_configuration(result_configuration)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn get_query_execution(
    client: &Client,
    execution_id: Option<impl Into<String>>,
) -> Result<GetQueryExecutionOutput, Error> {
    client
        .get_query_execution()
        .set_query_execution_id(execution_id.map(Into::into))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}
