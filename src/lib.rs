use crate::config::LSArgs;
use actix_web::{dev::Server, middleware::Logger, web::Data};
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use file_system::file_sync::{get_dir, get_file, get_root_uri};

use program::{
    code_runner::{kill_current_program, run_program_file},
    user_program::{UserProgram, UserProgramHandle},
};
use std::{
    net::TcpListener,
    sync::{atomic::AtomicBool, Arc},
};
use structopt::StructOpt;
use tokio::{process::Child, sync::Mutex};

pub mod config;
pub mod file_system;
pub mod language_server;
pub mod program;

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

pub fn get_ls_args() -> LSArgs {
    LSArgs::from_args()
}

pub struct AppState {
    pub ws_session_started: AtomicBool,
    pub lang: config::Lang,
    pub workspace_dir: String,
    pub program_input: Mutex<Vec<String>>,
    pub user_program: Arc<Mutex<Option<UserProgram>>>,
    pub user_program_handle: Arc<Mutex<Option<UserProgramHandle>>>,
}

pub fn run(
    listener: TcpListener,
    child: Arc<std::sync::Mutex<Child>>,
    state: Data<AppState>,
) -> Result<Server, std::io::Error> {
    println!("Program config: {:?}", LSArgs::from_args());
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let server = HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(state.clone())
            .service(
                web::scope("/code")
                    .route("/file/{filename:.*}", web::get().to(get_file))
                    .route("/directory", web::get().to(get_dir))
                    .route("/directory/root", web::get().to(get_root_uri))
                    .route("/run/{filename:.*}", web::get().to(run_program_file))
                    .route("/kill", web::get().to(kill_current_program)),
            )
            .route("/health", web::get().to(health_check))
            .data(child.clone())
            .route("/ls", web::route().to(language_server::server::to_lsp))
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
                    .route("/file/{filename:.*}", web::get().to(get_file))
                    .route("/directory", web::get().to(get_dir)),
            )
            .route("/health", web::get().to(health_check))
    })
    .listen(listener)?
    .run();
    Ok(server)
}
