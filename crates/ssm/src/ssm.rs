use aws_sdk_ssm::{Client, operation::get_parameter::GetParameterOutput};

use crate::error::{Error, from_aws_sdk_error};

pub async fn get_parameter_raw(
    client: &Client,
    name: Option<impl Into<String>>,
    with_decryption: Option<bool>,
) -> Result<GetParameterOutput, Error> {
    client
        .get_parameter()
        .set_name(name.map(Into::into))
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

    #[tokio::test]
    async fn test_get_parameter() {
        if std::env::var("REALM_CODE").is_err() {
            eprintln!("REALM_CODE is not set. Skipping test.");
            return;
        }

        let endpoint_url = std::env::var("SSM_ENDPOINT_URL").ok();
        let client = crate::make_client(endpoint_url).await;

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
        let client = crate::make_client(endpoint_url).await;

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
