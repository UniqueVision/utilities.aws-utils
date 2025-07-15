use crate::error::{Error, from_aws_sdk_error};
use aws_sdk_scheduler::{
    Client,
    operation::{
        create_schedule::CreateScheduleOutput, delete_schedule::DeleteScheduleOutput,
        update_schedule::UpdateScheduleOutput,
    },
    primitives::DateTime as AwsDateTime,
    types::{ActionAfterCompletion, FlexibleTimeWindow, ScheduleState, ScheduleSummary, Target},
};
use aws_smithy_types_convert::stream::PaginationStreamExt;
use chrono::prelude::*;
use futures_util::{Stream, TryStreamExt};

#[allow(clippy::too_many_arguments)]
pub async fn create_schedule(
    client: &Client,
    name: impl Into<String>,
    group_name: Option<impl Into<String>>,
    schedule_expression: impl Into<String>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
    description: Option<impl Into<String>>,
    schedule_expression_timezone: Option<impl Into<String>>,
    state: Option<ScheduleState>,
    kms_key_arn: Option<impl Into<String>>,
    target: Option<Target>,
    flexible_time_window: Option<FlexibleTimeWindow>,
    client_token: Option<impl Into<String>>,
    action_after_completion: Option<ActionAfterCompletion>,
) -> Result<CreateScheduleOutput, Error> {
    client
        .create_schedule()
        .name(name.into())
        .set_group_name(group_name.map(|g| g.into()))
        .schedule_expression(schedule_expression.into())
        .set_start_date(start_date.map(|d| AwsDateTime::from_millis(d.timestamp_millis())))
        .set_end_date(end_date.map(|d| AwsDateTime::from_millis(d.timestamp_millis())))
        .set_description(description.map(|d| d.into()))
        .set_schedule_expression_timezone(schedule_expression_timezone.map(|t| t.into()))
        .set_state(state)
        .set_kms_key_arn(kms_key_arn.map(|k| k.into()))
        .set_target(target)
        .set_flexible_time_window(flexible_time_window)
        .set_client_token(client_token.map(|c| c.into()))
        .set_action_after_completion(action_after_completion)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

#[allow(clippy::too_many_arguments)]
pub async fn update_schedule(
    client: &Client,
    name: impl Into<String>,
    group_name: Option<impl Into<String>>,
    schedule_expression: impl Into<String>,
    start_date: Option<DateTime<Utc>>,
    end_date: Option<DateTime<Utc>>,
    description: Option<impl Into<String>>,
    schedule_expression_timezone: Option<impl Into<String>>,
    state: Option<ScheduleState>,
    kms_key_arn: Option<impl Into<String>>,
    target: Option<Target>,
    flexible_time_window: Option<FlexibleTimeWindow>,
    client_token: Option<impl Into<String>>,
    action_after_completion: Option<ActionAfterCompletion>,
) -> Result<UpdateScheduleOutput, Error> {
    client
        .update_schedule()
        .name(name.into())
        .set_group_name(group_name.map(|g| g.into()))
        .schedule_expression(schedule_expression.into())
        .set_start_date(start_date.map(|d| AwsDateTime::from_millis(d.timestamp_millis())))
        .set_end_date(end_date.map(|d| AwsDateTime::from_millis(d.timestamp_millis())))
        .set_description(description.map(|d| d.into()))
        .set_schedule_expression_timezone(schedule_expression_timezone.map(|t| t.into()))
        .set_state(state)
        .set_kms_key_arn(kms_key_arn.map(|k| k.into()))
        .set_target(target)
        .set_flexible_time_window(flexible_time_window)
        .set_client_token(client_token.map(|c| c.into()))
        .set_action_after_completion(action_after_completion)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn delete_schedule(
    client: &Client,
    name: impl Into<String>,
    group_name: Option<impl Into<String>>,
    client_token: Option<impl Into<String>>,
) -> Result<DeleteScheduleOutput, Error> {
    client
        .delete_schedule()
        .name(name.into())
        .set_group_name(group_name.map(|g| g.into()))
        .set_client_token(client_token.map(|c| c.into()))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn get_scheduler(
    client: &Client,
    name: impl Into<String>,
    group_name: Option<impl Into<String>>,
) -> Result<aws_sdk_scheduler::operation::get_schedule::GetScheduleOutput, Error> {
    client
        .get_schedule()
        .name(name.into())
        .set_group_name(group_name.map(|g| g.into()))
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub fn list_schedules_stream(
    client: &Client,
    name_prefix: Option<impl Into<String>>,
    group_name: Option<impl Into<String>>,
    state: Option<ScheduleState>,
) -> impl Stream<Item = Result<ScheduleSummary, Error>> {
    client
        .list_schedules()
        .set_name_prefix(name_prefix.map(|n| n.into()))
        .set_group_name(group_name.map(|g| g.into()))
        .set_state(state)
        .into_paginator()
        .items()
        .send()
        .into_stream_03x()
        .map_err(from_aws_sdk_error)
}

pub async fn list_schedules_all(
    client: &Client,
    name_prefix: Option<impl Into<String>>,
    group_name: Option<impl Into<String>>,
    state: Option<ScheduleState>,
) -> Result<Vec<ScheduleSummary>, Error> {
    let stream = list_schedules_stream(client, name_prefix, group_name, state);
    futures_util::pin_mut!(stream);
    let mut result = vec![];
    while let Some(item) = stream.try_next().await? {
        result.push(item);
    }
    Ok(result)
}
