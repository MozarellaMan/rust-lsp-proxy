use lsp_proxy::config::LSArgs;
use lsp_proxy::run;
use std::{net::TcpListener, path::Path};
use structopt::StructOpt;

fn get_tcp_listener(port: i32) -> TcpListener {
    TcpListener::bind(format!("127.0.0.1:{}", port)).expect("failed to bind port to {}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = LSArgs::from_args();
    if !Path::new(&args.codebase_path).exists() {
        panic!("Directory does not exist!")
    }
    println!("Listening on {} ... 🚀", args.port);
    run(get_tcp_listener(args.port))?.await
}
