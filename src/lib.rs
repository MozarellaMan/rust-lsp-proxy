use crate::config::LSArgs;
use actix_web::{dev::Server, middleware::Logger};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;
use structopt::StructOpt;

pub mod code;
pub mod config;

// pub fn test_config() -> Option<LSArgs> {}

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    println!("Program config: {:?}", LSArgs::from_args());
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let server = HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(
                web::scope("/code")
                    .route("/file/{filename:.*}", web::get().to(code::get_file))
                    .route("/directory", web::get().to(health_check)),
            )
            .route("/health", web::get().to(health_check))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
