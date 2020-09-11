use lsp_proxy::config::LSArgs;
use lsp_proxy::run;
use std::net::TcpListener;
use structopt::StructOpt;

fn get_tcp_listener(port: i32) -> TcpListener {
    let listener =
        TcpListener::bind(format!("127.0.0.1:{}", port)).expect("failed to bind port to {}");
    listener
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = LSArgs::from_args();
    println!("Listening on {} ... 🚀", args.port);
    run(get_tcp_listener(args.port))?.await
}
