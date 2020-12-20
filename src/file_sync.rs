use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};
use serde::Deserialize;

#[derive(Deserialize)]
pub enum FileSyncType {
    New,
    Update,
    Delete,
}
#[derive(Deserialize)]
pub struct FileSyncMsg {
    pub reason: FileSyncType,
    pub name: String,
    pub text: String,
}

#[derive(Debug, Display, Error)]
pub enum FileSyncError {
    #[display(fmt = "internal error {}", cause)]
    InternalError { cause: String },

    #[display(fmt = "bad request {}", cause)]
    BadClientData { cause: String },

    #[display(fmt = "timeout")]
    Timeout,

    #[display(fmt = "file not found")]
    NotFound,
}

pub fn map_io_err(e: std::io::Error) -> FileSyncError {
    match e.kind() {
        std::io::ErrorKind::NotFound => FileSyncError::NotFound,
        _ => FileSyncError::InternalError {
            cause: e.to_string(),
        },
    }
}

impl error::ResponseError for FileSyncError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            FileSyncError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            FileSyncError::BadClientData { .. } => StatusCode::BAD_REQUEST,
            FileSyncError::Timeout => StatusCode::GATEWAY_TIMEOUT,
            FileSyncError::NotFound => StatusCode::NOT_FOUND,
        }
    }
}
