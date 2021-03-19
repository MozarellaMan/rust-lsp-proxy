use std::{env, net::TcpListener};

use lsp_proxy::config::{Lang, LsArgs};

pub const COMMON_TEST_DIRECTORY: &str = "./tests/integration/example_code_repos/test-java-repo";
pub const COMMON_TEST_FILE: &str =
    "./tests/integration/example_code_repos/test-java-repo/src/Hello.java";
pub const COMMON_TEST_LANG: Lang = lsp_proxy::config::Lang::Java;
const TEST_JAVA_SERVER_PATH: &str = "./";

fn setup_program_args(args: &LsArgs) {
    env::set_var("CODEBASE_PATH", &args.codebase_path);
    env::set_var("PORT", &args.port.to_string());
    env::set_var("LANGUAGE", &args.language.to_string());
    env::set_var("LANG_SERVER_PATH", &args.lang_server_path.to_string());
}

pub fn spawn_app(codebase_path: &str, language: Lang) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    // retrieve OS assigned port
    let port = listener.local_addr().unwrap().port();
    let args = LsArgs {
        codebase_path: codebase_path.into(),
        port: port.into(),
        language,
        lang_server_path: TEST_JAVA_SERVER_PATH.into(),
        custom_lang_server_cmd: None,
    };

    // include_str!("./example_code_repos/test-java-repo/src/Hello.java");
    setup_program_args(&args);

    println!("test port: {}", port);
    let server = lsp_proxy::test_run(listener).expect("failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
