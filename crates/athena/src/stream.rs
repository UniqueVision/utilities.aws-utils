use std::time::Duration;

use aws_sdk_athena::{
    Client,
    operation::get_query_execution::GetQueryExecutionOutput,
    types::{QueryExecutionContext, QueryExecutionState, ResultConfiguration, ResultSet},
};
use futures_util::{Stream, stream::unfold};

use crate::{
    error::Error,
    query::{get_query_execution, get_query_results, start_query_execution},
};

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

pub async fn execute_query_for_stream(
    client: &Client,
    query_string: Option<impl Into<String>>,
    query_execution_context: Option<QueryExecutionContext>,
    result_configuration: Option<ResultConfiguration>,
    timeout_duration: Duration,
    check_duration: Duration,
) -> Result<String, Error> {
    let query_execution_id = start_query_execution(
        client,
        query_string,
        query_execution_context,
        result_configuration,
    )
    .await?;
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

pub fn get_query_results_stream(
    client: &Client,
    execution_id: String,
) -> impl Stream<Item = Result<ResultSet, Error>> {
    Box::pin(unfold(
        (client, None, execution_id),
        |(client, next_token, execution_id): (&Client, Option<String>, String)| async move {
            if let Some(ref next_token) = next_token
                && next_token.is_empty()
            {
                return None;
            }
            match get_query_results(client, Some(&execution_id), next_token).await {
                Ok(result) => {
                    // ここでnext_tokenがNoneの場合は終了。終了条件はNoneでは無く空文字列
                    let next_token = match result.next_token() {
                        Some(token) => Some(token.to_owned()),
                        None => Some("".to_owned()),
                    };
                    match result.result_set() {
                        Some(result_set) => {
                            Some((Ok(result_set.clone()), (client, next_token, execution_id)))
                        }
                        None => Some((
                            Err(Error::Invalid("result set is invalid".to_owned())),
                            (client, Some("".to_owned()), execution_id),
                        )),
                    }
                }
                Err(e) => Some((Err(e), (client, Some("".to_owned()), execution_id))),
            }
        },
    ))
}
