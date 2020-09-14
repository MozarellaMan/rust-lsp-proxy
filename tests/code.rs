mod test_helper;

use reqwest::StatusCode;
use std::fs;
use test_helper::{COMMON_TEST_DIRECTORY, COMMON_TEST_LANG, _COMMON_TEST_FILE};

#[actix_rt::test]
async fn file_endpoint_responds_with_existing_file() {
    // Arrange
    let address = test_helper::spawn_app(COMMON_TEST_DIRECTORY, COMMON_TEST_LANG);
    let client = reqwest::Client::new();
    let actual_file = fs::read_to_string(_COMMON_TEST_FILE).ok();
    let input_path = "src/Hello.java";

    let response = client
        .get(&format!("{}/code/file/{}", &address, input_path))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let content = response.text().await.expect("failed to get req content");

    assert_eq!(actual_file.unwrap(), content.as_str());
}
