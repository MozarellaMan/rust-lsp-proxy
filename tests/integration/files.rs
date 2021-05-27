use crate::test_helper::{spawn_app, COMMON_TEST_DIRECTORY, COMMON_TEST_LANG};
use assert_json_diff::assert_json_eq;
use reqwest::StatusCode;
use serde_json::{json, Value};

#[actix_rt::test]
async fn check_directory_endpoint_returns_correct_json() {
    // Arrange test components
    let address = spawn_app(COMMON_TEST_DIRECTORY, COMMON_TEST_LANG);
    let client = reqwest::Client::new();

    // expected JSON data
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
            },
            {
                "path": format!("{}/src/Terro.java", COMMON_TEST_DIRECTORY),
                "name": "Terro.java",
                "type": "java",
                "children": [],
            }
            ],
        }]
    });

    // Act
    let response = client
        .get(&format!("{}/code/directory", &address))
        .send()
        .await
        .expect("failed to execute request");

    // assert OK response from proxy
    assert_eq!(response.status(), StatusCode::OK);

    // check return value matches the expected data
    let response = response.json::<Value>().await;
    let response = response.unwrap_or_default();

    println!("{:#}", response);
    println!("{:#}", expected);
    assert_json_eq!(response, expected);
}
