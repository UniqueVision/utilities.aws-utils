use crate::error::{Error, from_aws_sdk_error};
use aws_sdk_scheduler::{
    Client,
    operation::{create_schedule::CreateScheduleOutput, delete_schedule::DeleteScheduleOutput},
    primitives::DateTime as AwsDateTime,
    types::{ActionAfterCompletion, FlexibleTimeWindow, ScheduleState, Target},
};
use chrono::prelude::*;

#[allow(clippy::too_many_arguments)]
pub async fn create_scheduler(
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

pub async fn delete_scheduler(
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

