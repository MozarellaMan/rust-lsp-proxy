mod test_helper;

use assert_json_diff::assert_json_eq;
use reqwest::StatusCode;
use serde_json::{Value, json};
use test_helper::{COMMON_TEST_LANG, _COMMON_TEST_DIRECTORY};

#[actix_rt::test]
async fn directory_tree_json() {
    // Arrange
    let address = test_helper::spawn_app(_COMMON_TEST_DIRECTORY, COMMON_TEST_LANG);
    let client = reqwest::Client::new();
    let expected = json!({
        "path": "",
        "name": "test-java-repo",
        "type": "directory",
        "children": [{
            "path": "src",
            "name": "src",
            "type": "directory",
            "children": [{
                "path": "src/Hello.java",
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
