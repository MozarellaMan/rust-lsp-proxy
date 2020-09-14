use std::net::TcpListener;

pub fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    // retrieve OS assigned port
    let port = listener.local_addr().unwrap().port();
    println!("test port: {}", port);
    let server = lsp_proxy::run(listener).expect("failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
