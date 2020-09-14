use std::fs;

mod test_helper;

const TEST_FILE: &str = "tests/example_code_repos/test-java-repo/src/Hello.java";

#[actix_rt::test]
async fn file_endpoint_responds_with_existing_file() {
    // Arrange
    let address = test_helper::spawn_app();
    let client = reqwest::Client::new();
    let actual_file = fs::read_to_string(TEST_FILE).ok();

    let response = client
        .get(&format!("{}/file/{}", &address, TEST_FILE))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());

    let content = response.text().await.expect("failed to get req content");

    assert_eq!(actual_file.unwrap(), content.as_str());
}
