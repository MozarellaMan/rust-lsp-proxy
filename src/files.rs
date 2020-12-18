use actix_web::{HttpResponse, Responder};


pub async fn get_dir() -> impl Responder {
    HttpResponse::Ok()
}