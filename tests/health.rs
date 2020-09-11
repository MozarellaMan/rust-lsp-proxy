//use actix_web::test;
use std::net::TcpListener;

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
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

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    // retrieve OS assigned port
    let port = listener.local_addr().unwrap().port();
    let server = lsp_proxy::run(listener).expect("failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
