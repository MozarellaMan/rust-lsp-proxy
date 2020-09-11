use crate::config::LSArgs;
use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::net::TcpListener;
use structopt::StructOpt;

pub mod config;

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    println!("{:?}", LSArgs::from_args());
    let server = HttpServer::new(|| App::new().route("/health", web::get().to(health_check)))
        .listen(listener)?
        .run();
    Ok(server)
}
