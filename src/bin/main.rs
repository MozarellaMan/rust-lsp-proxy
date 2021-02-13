use actix_web::web;
use lsp_proxy::{
    config::{LSArgs, Lang},
    language_server::server::start_lang_server,
};
use lsp_proxy::{run, AppState};
use std::{
    net::TcpListener,
    path::Path,
    sync::{atomic::AtomicBool, Arc},
};
use structopt::StructOpt;
use tokio::sync::Mutex;

fn get_tcp_listener(port: i32) -> TcpListener {
    TcpListener::bind(format!("127.0.0.1:{}", port)).expect("failed to bind port to {}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = LSArgs::from_args();
    if !Path::new(&args.codebase_path).exists() {
        panic!("Directory does not exist!")
    }
    let path: String = args.codebase_path;

    let child = start_lang_server(Lang::Java, path.clone()).unwrap();
    println!("Listening on {} ... 🚀", args.port);

    let state = web::Data::new(AppState {
        ws_session_started: AtomicBool::from(false),
        lang: args.language,
        workspace_dir: path,
        program_input: Mutex::new(Vec::new()),
        running_program: Arc::new(Mutex::new(None)),
    });
    run(
        get_tcp_listener(args.port),
        Arc::new(std::sync::Mutex::new(child)),
        state,
    )?
    .await
}
