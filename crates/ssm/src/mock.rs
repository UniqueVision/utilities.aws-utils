use aws_sdk_ssm::{
    operation::get_parameter::GetParameterOutput,
    types::{Parameter, ParameterType},
};

pub const MOCK_ENV_PREFIX: &str = "SSM_MOCK_";

/// パラメータ名をmock用の環境変数名に変換する
/// 例: `/app/db/host` → `SSM_MOCK_APP_DB_HOST`
pub fn mock_env_name(name: &str) -> String {
    let converted: String = name
        .trim_start_matches('/')
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect();
    format!("{MOCK_ENV_PREFIX}{converted}")
}

/// mock用の環境変数が設定されていればGetParameterOutputを組み立てて返す
pub(crate) fn get_parameter_output(name: &str) -> Option<GetParameterOutput> {
    let value = std::env::var(mock_env_name(name)).ok()?;
    let parameter = Parameter::builder()
        .name(name)
        .value(value)
        .r#type(ParameterType::String)
        .build();
    Some(GetParameterOutput::builder().parameter(parameter).build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ssm::{get_parameter, get_parameter_raw};

    #[test]
    fn test_mock_env_name() {
        assert_eq!(mock_env_name("/app/db/host"), "SSM_MOCK_APP_DB_HOST");
        assert_eq!(mock_env_name("app/db-name.v1"), "SSM_MOCK_APP_DB_NAME_V1");
    }

    #[tokio::test]
    async fn test_get_parameter_mock() {
        // SAFETY: このテスト専用の環境変数のみを操作する
        unsafe {
            std::env::set_var("SSM_MOCK_TEST_MOCK_PARAMETER", "mock-value");
        }
        let client = crate::make_client(None, None, None).await;

        let value = get_parameter(&client, "/test/mock/parameter")
            .await
            .unwrap();
        assert_eq!(value, "mock-value");

        let output = get_parameter_raw(&client, Some("/test/mock/parameter"), Some(true))
            .await
            .unwrap();
        let param = output.parameter().unwrap();
        assert_eq!(param.name(), Some("/test/mock/parameter"));
        assert_eq!(param.value(), Some("mock-value"));
    }
}
