mod test_helper;

use test_helper::{_COMMON_TEST_DIRECTORY, COMMON_TEST_LANG};

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let address = test_helper::spawn_app(_COMMON_TEST_DIRECTORY, COMMON_TEST_LANG);
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health", &address))
        .send()
        .await
        .expect("failed to execute request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
