use crate::AppState;
use actix_web::{error::ErrorBadRequest, web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use server::LangServer;
use std::sync::{atomic::Ordering, Arc};
use tokio::process::Child;

pub mod intercept;
pub mod server;
pub mod server_runners;

pub async fn to_language_server(
    req: HttpRequest,
    stream: web::Payload,
    process: web::Data<Arc<std::sync::Mutex<Child>>>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    let session_started = state.ws_session_started.load(Ordering::Relaxed);
    if !session_started {
        let lang_server = LangServer::new(process.as_ref().to_owned());
        let session = ws::start(lang_server, &req, stream);
        println!("Language Server started\n{:?}", session);
        state.ws_session_started.store(true, Ordering::Relaxed);
        session
    } else {
        Err(ErrorBadRequest(
            "Language server web socket session already started.",
        ))
    }
}
