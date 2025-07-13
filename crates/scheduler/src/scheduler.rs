use aws_sdk_scheduler::{operation::{create_schedule::CreateScheduleOutput}, primitives::DateTime, Client};
use crate::error::{Error, from_aws_sdk_error};

pub async fn create_scheduler(
    client: &Client,
    name: impl Into<String>,
    group_name: Option<impl Into<String>>,
    schedule_expression: impl Into<String>,
    start_date: Option<DateTime>,
    end_date: Option<DateTime>,
) -> Result<CreateScheduleOutput, Error> {
    client
        .create_schedule()
        .name(name.into())
        .set_group_name(group_name.map(|g| g.into()))
        .schedule_expression(schedule_expression.into())
        .set_start_date(start_date)
        .set_end_date(end_date)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}
