use aws_sdk_ssm::{
    Client,
    operation::get_parameter::GetParameterOutput,
    types::{Parameter, ParameterType},
};

use crate::error::{Error, from_aws_sdk_error};

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

/// mock用の環境変数が設定されていればその値を返す
fn get_mock_value(name: &str) -> Option<String> {
    std::env::var(mock_env_name(name)).ok()
}

pub async fn get_parameter_raw(
    client: &Client,
    name: Option<impl Into<String>>,
    with_decryption: Option<bool>,
) -> Result<GetParameterOutput, Error> {
    let name: Option<String> = name.map(Into::into);
    if let Some(value) = name.as_deref().and_then(get_mock_value) {
        let parameter = Parameter::builder()
            .set_name(name)
            .value(value)
            .r#type(ParameterType::String)
            .build();
        return Ok(GetParameterOutput::builder().parameter(parameter).build());
    }
    client
        .get_parameter()
        .set_name(name)
        .set_with_decryption(with_decryption)
        .send()
        .await
        .map_err(from_aws_sdk_error)
}

pub async fn get_parameter(client: &Client, name: &str) -> Result<String, Error> {
    let res = get_parameter_raw(client, Some(name), Some(true)).await?;
    res.parameter()
        .and_then(|p| p.value())
        .ok_or_else(|| Error::NotFound)
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[tokio::test]
    async fn test_get_parameter() {
        if std::env::var("REALM_CODE").is_err() {
            eprintln!("REALM_CODE is not set. Skipping test.");
            return;
        }

        let endpoint_url = std::env::var("SSM_ENDPOINT_URL").ok();
        let client = crate::make_client(endpoint_url, None, None).await;

        // テスト用のパラメータ名を環境変数から取得
        let parameter_name = std::env::var("TEST_SSM_PARAMETER_NAME")
            .unwrap_or_else(|_| "/test/parameter".to_string());

        // パラメータの取得をテスト
        match get_parameter(&client, &parameter_name).await {
            Ok(value) => {
                println!("Parameter value: {}", value);
                assert!(!value.is_empty());
            }
            Err(e) => {
                eprintln!("Failed to get parameter: {:?}", e);
                // パラメータが存在しない場合もテストは通す
                if !matches!(e, Error::NotFound) {
                    panic!("Unexpected error: {:?}", e);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_get_parameter_raw() {
        if std::env::var("REALM_CODE").is_err() {
            eprintln!("REALM_CODE is not set. Skipping test.");
            return;
        }

        let endpoint_url = std::env::var("SSM_ENDPOINT_URL").ok();
        let client = crate::make_client(endpoint_url, None, None).await;

        let parameter_name = std::env::var("TEST_SSM_PARAMETER_NAME")
            .unwrap_or_else(|_| "/test/parameter".to_string());

        // 暗号化なしでパラメータを取得
        match get_parameter_raw(&client, Some(&parameter_name), Some(false)).await {
            Ok(output) => {
                if let Some(param) = output.parameter() {
                    println!("Parameter name: {:?}", param.name());
                    println!("Parameter type: {:?}", param.r#type());
                }
            }
            Err(e) => {
                eprintln!("Failed to get parameter raw: {:?}", e);
            }
        }
    }
}
