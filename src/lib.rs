use crate::config::LSArgs;
use actix_web::{dev::Server, middleware::Logger};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use lang_server::to_lsp;
use std::{net::TcpListener, sync::Arc};
use structopt::StructOpt;
use tokio::process::Child;

pub mod code;
pub mod config;
pub mod lang_server;
pub mod files;

// pub fn test_config() -> Option<LSArgs> {}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn get_ls_args() -> LSArgs {
    LSArgs::from_args()
}

pub fn run(
    listener: TcpListener,
    child: Arc<std::sync::Mutex<Child>>,
) -> Result<Server, std::io::Error> {
    println!("Program config: {:?}", LSArgs::from_args());
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(
                web::scope("/code")
                    .route("/file/{filename:.*}", web::get().to(code::get_file))
                    .route("/directory", web::get().to(files::get_dir)),
            )
            .route("/health", web::get().to(health_check))
            .data(child.clone())
            .route("/ls", web::route().to(to_lsp))
    })
    .listen(listener)?
    .run();
    Ok(server)
}

pub fn test_run(listener: TcpListener) -> Result<Server, std::io::Error> {
    println!("Program config: {:?}", LSArgs::from_args());
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(
                web::scope("/code")
                    .route("/file/{filename:.*}", web::get().to(code::get_file))
                    .route("/directory", web::get().to(files::get_dir)),
            )
            .route("/health", web::get().to(health_check))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
