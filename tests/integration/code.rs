use crate::test_helper::{spawn_app, COMMON_TEST_DIRECTORY, COMMON_TEST_FILE, COMMON_TEST_LANG};
use reqwest::StatusCode;
use std::{env, fs};

#[actix_rt::test]
async fn file_endpoint_responds_with_existing_file() {
    // Arrange
    let address = spawn_app(COMMON_TEST_DIRECTORY, COMMON_TEST_LANG);
    let client = reqwest::Client::new();
    let path = env::current_dir().expect("no dir");
    println!("The current directory is {}", path.display());
    let actual_file = fs::read_to_string(COMMON_TEST_FILE).expect("cannot find test file!");
    let input_path = "src/Hello.java";

    let response = client
        .get(&format!("{}/code/file/{}", &address, input_path))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);

    let content = response.text().await.expect("failed to get req content");

    assert_eq!(actual_file, content.as_str());
}
