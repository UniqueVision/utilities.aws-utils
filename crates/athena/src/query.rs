use aws_sdk_athena::{
    Client,
    operation::{
        get_query_execution::GetQueryExecutionOutput,
        start_query_execution::StartQueryExecutionOutput,
    },
    types::{QueryExecutionContext, ResultConfiguration, ResultReuseConfiguration, ResultSet},
};
use aws_smithy_types_convert::stream::PaginationStreamExt;
use futures_util::{TryStream, TryStreamExt};

use crate::error::{Error, from_aws_sdk_error};

#[allow(clippy::too_many_arguments)]
pub async fn start_query_execution(
    client: &Client,
    query_string: Option<impl Into<String>>,
    query_execution_context: Option<QueryExecutionContext>,
    result_configuration: Option<ResultConfiguration>,
    client_request_token: Option<impl Into<String>>,
    execution_parameters: Option<Vec<String>>,
    result_reuse_configuration: Option<ResultReuseConfiguration>,
    work_group: Option<impl Into<String>>,
) -> Result<StartQueryExecutionOutput, Error> {
    client
        .start_query_execution()
        .set_query_string(query_string.map(Into::into))
        .set_query_execution_context(query_execution_context)
        .set_result_configuration(result_configuration)
        .set_client_request_token(client_request_token.map(Into::into))
        .set_execution_parameters(execution_parameters)
        .set_result_reuse_configuration(result_reuse_configuration)
        .set_work_group(work_group.map(Into::into))
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

pub fn get_query_results_stream(
    client: &Client,
    execution_id: Option<impl Into<String>>,
) -> impl TryStream<Ok = ResultSet, Error = Error> {
    client
        .get_query_results()
        .set_query_execution_id(execution_id.map(Into::into))
        .into_paginator()
        .send()
        .into_stream_03x()
        .map_err(from_aws_sdk_error)
        .and_then(|s| async move {
            s.result_set()
                .ok_or_else(|| Error::Invalid("result_set is None".to_string()))
                .cloned()
        })
}
