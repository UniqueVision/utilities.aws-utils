use crate::error::Error;
use chrono::{DateTime, Utc};

/// Builder for one-time schedule expressions
/// Format: at(yyyy-mm-ddThh:mm:ss)
pub struct AtExpressionBuilder {
    datetime: Option<DateTime<Utc>>,
}

impl AtExpressionBuilder {
    pub fn new() -> Self {
        Self { datetime: None }
    }

    pub fn datetime(mut self, datetime: DateTime<Utc>) -> Self {
        self.datetime = Some(datetime);
        self
    }

    pub fn build(&self) -> Result<String, Error> {
        let datetime = self.datetime.ok_or_else(|| {
            Error::ValidationError("datetime is required for at expression".to_string())
        })?;

        Ok(format!("at({})", datetime.format("%Y-%m-%dT%H:%M:%S")))
    }
}

impl Default for AtExpressionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Time unit for rate-based schedules
#[derive(Debug, Clone, Copy)]
pub enum RateUnit {
    Minutes,
    Hours,
    Days,
}

impl RateUnit {
    fn as_str(&self, value: u32) -> &'static str {
        match self {
            RateUnit::Minutes => {
                if value == 1 {
                    "minute"
                } else {
                    "minutes"
                }
            }
            RateUnit::Hours => {
                if value == 1 {
                    "hour"
                } else {
                    "hours"
                }
            }
            RateUnit::Days => {
                if value == 1 {
                    "day"
                } else {
                    "days"
                }
            }
        }
    }
}

/// Builder for rate-based schedule expressions
/// Format: rate(value unit)
pub struct RateExpressionBuilder {
    value: Option<u32>,
    unit: Option<RateUnit>,
}

impl RateExpressionBuilder {
    pub fn new() -> Self {
        Self {
            value: None,
            unit: None,
        }
    }

    pub fn value(mut self, value: u32) -> Self {
        self.value = Some(value);
        self
    }

    pub fn unit(mut self, unit: RateUnit) -> Self {
        self.unit = Some(unit);
        self
    }

    pub fn minutes(mut self, value: u32) -> Self {
        self.value = Some(value);
        self.unit = Some(RateUnit::Minutes);
        self
    }

    pub fn hours(mut self, value: u32) -> Self {
        self.value = Some(value);
        self.unit = Some(RateUnit::Hours);
        self
    }

    pub fn days(mut self, value: u32) -> Self {
        self.value = Some(value);
        self.unit = Some(RateUnit::Days);
        self
    }

    pub fn build(&self) -> Result<String, Error> {
        let value = self.value.ok_or_else(|| {
            Error::ValidationError("value is required for rate expression".to_string())
        })?;

        if value == 0 {
            return Err(Error::ValidationError(
                "value must be a positive number".to_string(),
            ));
        }

        let unit = self.unit.ok_or_else(|| {
            Error::ValidationError("unit is required for rate expression".to_string())
        })?;

        Ok(format!("rate({} {})", value, unit.as_str(value)))
    }
}

impl Default for RateExpressionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for cron-based schedule expressions
/// Format: cron(minutes hours day-of-month month day-of-week year)
pub struct CronExpressionBuilder {
    minutes: Option<String>,
    hours: Option<String>,
    day_of_month: Option<String>,
    month: Option<String>,
    day_of_week: Option<String>,
    year: Option<String>,
}

impl CronExpressionBuilder {
    pub fn new() -> Self {
        Self {
            minutes: None,
            hours: None,
            day_of_month: None,
            month: None,
            day_of_week: None,
            year: None,
        }
    }

    pub fn minutes(mut self, minutes: impl Into<String>) -> Self {
        self.minutes = Some(minutes.into());
        self
    }

    pub fn hours(mut self, hours: impl Into<String>) -> Self {
        self.hours = Some(hours.into());
        self
    }

    pub fn day_of_month(mut self, day_of_month: impl Into<String>) -> Self {
        self.day_of_month = Some(day_of_month.into());
        self
    }

    pub fn month(mut self, month: impl Into<String>) -> Self {
        self.month = Some(month.into());
        self
    }

    pub fn day_of_week(mut self, day_of_week: impl Into<String>) -> Self {
        self.day_of_week = Some(day_of_week.into());
        self
    }

    pub fn year(mut self, year: impl Into<String>) -> Self {
        self.year = Some(year.into());
        self
    }

    fn validate_field(field: &str, name: &str, min: i32, max: i32) -> Result<(), Error> {
        // Skip validation for wildcards and special characters
        if field == "*"
            || field == "?"
            || field.contains(',')
            || field.contains('-')
            || field.contains('/')
            || field.contains('L')
            || field.contains('W')
            || field.contains('#')
        {
            return Ok(());
        }

        // Try to parse as number
        if let Ok(value) = field.parse::<i32>() {
            if value < min || value > max {
                return Err(Error::ValidationError(format!(
                    "{name} must be between {min} and {max}"
                )));
            }
        }

        Ok(())
    }

    pub fn build(&self) -> Result<String, Error> {
        let minutes = self.minutes.as_ref().ok_or_else(|| {
            Error::ValidationError("minutes is required for cron expression".to_string())
        })?;
        let hours = self.hours.as_ref().ok_or_else(|| {
            Error::ValidationError("hours is required for cron expression".to_string())
        })?;
        let day_of_month = self.day_of_month.as_ref().ok_or_else(|| {
            Error::ValidationError("day_of_month is required for cron expression".to_string())
        })?;
        let month = self.month.as_ref().ok_or_else(|| {
            Error::ValidationError("month is required for cron expression".to_string())
        })?;
        let day_of_week = self.day_of_week.as_ref().ok_or_else(|| {
            Error::ValidationError("day_of_week is required for cron expression".to_string())
        })?;

        // Validate fields
        Self::validate_field(minutes, "minutes", 0, 59)?;
        Self::validate_field(hours, "hours", 0, 23)?;
        Self::validate_field(day_of_month, "day_of_month", 1, 31)?;

        // Validate month (1-12 or JAN-DEC)
        if !month
            .chars()
            .all(|c| c.is_alphabetic() || c == '-' || c == ',' || c == '*' || c == '?')
        {
            Self::validate_field(month, "month", 1, 12)?;
        }

        // Validate day_of_week (1-7 or SUN-SAT)
        if !day_of_week
            .chars()
            .all(|c| c.is_alphabetic() || c == '-' || c == ',' || c == '*' || c == '?')
        {
            Self::validate_field(day_of_week, "day_of_week", 1, 7)?;
        }

        // Validate year if provided
        if let Some(year) = &self.year {
            Self::validate_field(year, "year", 1970, 2199)?;
        }

        // Cannot specify both day_of_month and day_of_week
        if day_of_month != "?" && day_of_week != "?" {
            return Err(Error::ValidationError(
                "Cannot specify both day_of_month and day_of_week, one must be '?'".to_string(),
            ));
        }

        let expression = if let Some(year) = &self.year {
            format!("cron({minutes} {hours} {day_of_month} {month} {day_of_week} {year})")
        } else {
            format!("cron({minutes} {hours} {day_of_month} {month} {day_of_week})")
        };

        Ok(expression)
    }
}

impl Default for CronExpressionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_at_expression_builder() {
        let datetime = Utc.with_ymd_and_hms(2022, 11, 20, 13, 0, 0).unwrap();
        let expression = AtExpressionBuilder::new()
            .datetime(datetime)
            .build()
            .unwrap();

        assert_eq!(expression, "at(2022-11-20T13:00:00)");
    }

    #[test]
    fn test_at_expression_builder_missing_datetime() {
        let result = AtExpressionBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_rate_expression_builder() {
        let expression = RateExpressionBuilder::new()
            .value(5)
            .unit(RateUnit::Minutes)
            .build()
            .unwrap();

        assert_eq!(expression, "rate(5 minutes)");
    }

    #[test]
    fn test_rate_expression_builder_singular() {
        let expression = RateExpressionBuilder::new().hours(1).build().unwrap();

        assert_eq!(expression, "rate(1 hour)");
    }

    #[test]
    fn test_rate_expression_builder_convenience_methods() {
        let expression = RateExpressionBuilder::new().days(7).build().unwrap();

        assert_eq!(expression, "rate(7 days)");
    }

    #[test]
    fn test_rate_expression_builder_zero_value() {
        let result = RateExpressionBuilder::new()
            .value(0)
            .unit(RateUnit::Hours)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_cron_expression_builder() {
        let expression = CronExpressionBuilder::new()
            .minutes("15")
            .hours("10")
            .day_of_month("?")
            .month("*")
            .day_of_week("6L")
            .year("2022-2023")
            .build()
            .unwrap();

        assert_eq!(expression, "cron(15 10 ? * 6L 2022-2023)");
    }

    #[test]
    fn test_cron_expression_builder_without_year() {
        let expression = CronExpressionBuilder::new()
            .minutes("0")
            .hours("12")
            .day_of_month("1")
            .month("*")
            .day_of_week("?")
            .build()
            .unwrap();

        assert_eq!(expression, "cron(0 12 1 * ?)");
    }

    #[test]
    fn test_cron_expression_builder_with_names() {
        let expression = CronExpressionBuilder::new()
            .minutes("30")
            .hours("14")
            .day_of_month("?")
            .month("JAN,JUL")
            .day_of_week("MON-FRI")
            .build()
            .unwrap();

        assert_eq!(expression, "cron(30 14 ? JAN,JUL MON-FRI)");
    }

    #[test]
    fn test_cron_expression_builder_invalid_both_day_fields() {
        let result = CronExpressionBuilder::new()
            .minutes("0")
            .hours("12")
            .day_of_month("15")
            .month("*")
            .day_of_week("MON")
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_cron_expression_builder_invalid_minute() {
        let result = CronExpressionBuilder::new()
            .minutes("60")
            .hours("12")
            .day_of_month("?")
            .month("*")
            .day_of_week("*")
            .build();

        assert!(result.is_err());
    }
}
