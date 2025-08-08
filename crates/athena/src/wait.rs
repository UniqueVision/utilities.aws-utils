use std::time::Duration;

use aws_sdk_athena::{
    Client,
    operation::{
        get_query_execution::GetQueryExecutionOutput,
        start_query_execution::builders::StartQueryExecutionFluentBuilder,
    },
    types::QueryExecutionState,
};

use crate::{
    error::{Error, from_aws_sdk_error},
    query::get_query_execution,
};

pub async fn start_query_execution_wait(
    client: &Client,
    builder: StartQueryExecutionFluentBuilder,
    timeout_duration: Duration,
    check_duration: Duration,
) -> Result<String, Error> {
    let query_execution_id = builder.send().await.map_err(from_aws_sdk_error)?;
    let query_execution_id = query_execution_id
        .query_execution_id()
        .ok_or_else(|| Error::Invalid("query execution ID is missing".to_owned()))?
        .to_string();

    tokio::time::timeout(
        timeout_duration,
        check_query_succeeded(client, &query_execution_id, check_duration),
    )
    .await??;

    Ok(query_execution_id)
}

async fn check_query_succeeded(
    client: &Client,
    query_execution_id: &str,
    duration: Duration,
) -> Result<(), Error> {
    loop {
        let get_query_execution = get_query_execution(client, Some(query_execution_id)).await?;
        if inner_check_query_succeeded(&get_query_execution)? {
            return Ok(());
        };
        tokio::time::sleep(duration).await;
    }
}

fn inner_check_query_succeeded(
    get_query_execution: &GetQueryExecutionOutput,
) -> Result<bool, Error> {
    let Some(query_execution) = get_query_execution.query_execution() else {
        return Err(Error::Invalid("query execution is invalid".to_owned()));
    };

    match query_execution.status() {
        Some(status) => {
            if let Some(state) = status.state() {
                match state {
                    QueryExecutionState::Succeeded => Ok(true),
                    QueryExecutionState::Cancelled => Err(Error::QueryCancelled),
                    QueryExecutionState::Failed => Err(Error::QueryFailed),
                    QueryExecutionState::Queued => Ok(false),
                    QueryExecutionState::Running => Ok(false),
                    _ => Ok(false),
                }
            } else {
                Err(Error::Invalid("query state is invalid".to_owned()))
            }
        }
        None => Err(Error::Invalid("query state is invalid".to_owned())),
    }
}
