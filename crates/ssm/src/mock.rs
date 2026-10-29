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

    /// 子プロセスとして実行されたことを示す環境変数
    const CHILD_ENV: &str = "AWS_UTILS_SSM_MOCK_TEST_CHILD";
    const CHILD_TEST: &str = "mock::tests::get_parameter_mock_child";

    #[test]
    fn test_get_parameter_mock() {
        // 並列実行中の他テストが環境変数を読むため、set_varは未定義動作になり得る。
        // 環境変数を渡した子プロセスでテストバイナリを再実行して検証する
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
            .env(CHILD_ENV, "1")
            .env("SSM_MOCK_TEST_MOCK_PARAMETER", "mock-value")
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "child failed:\n{stdout}\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        // フィルタが一致せず0件で成功する事態を防ぐ
        assert!(
            stdout.contains("1 passed"),
            "child test did not run:\n{stdout}"
        );
    }

    #[tokio::test]
    async fn get_parameter_mock_child() {
        if std::env::var(CHILD_ENV).is_err() {
            return;
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
