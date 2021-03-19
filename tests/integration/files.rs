use assert_json_diff::assert_json_eq;
use reqwest::StatusCode;
use serde_json::{json, Value};
use crate::test_helper::{COMMON_TEST_LANG, COMMON_TEST_DIRECTORY, spawn_app};

#[actix_rt::test]
async fn directory_tree_json() {
    // Arrange
    let address = spawn_app(COMMON_TEST_DIRECTORY, COMMON_TEST_LANG);
    let client = reqwest::Client::new();
    let expected = json!({
        "path": COMMON_TEST_DIRECTORY,
        "name": "test-java-repo",
        "type": "directory",
        "children": [{
            "path": format!("{}/src", COMMON_TEST_DIRECTORY),
            "name": "src",
            "type": "directory",
            "children": [{
                "path": format!("{}/src/Hello.java", COMMON_TEST_DIRECTORY),
                "name": "Hello.java",
                "type": "java",
                "children": [],
            }],
        }]
    });

    // Act
    let response = client
        .get(&format!("{}/code/directory", &address))
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(response.status(), StatusCode::OK);
    let response = response.json::<Value>().await;
    assert_json_eq!(response.unwrap_or_default(), expected);
}
