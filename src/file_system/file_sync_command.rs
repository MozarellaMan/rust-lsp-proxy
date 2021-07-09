use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};

pub enum FileSyncType {
    New,
    Update,
    Delete,
}
pub struct FileSyncCommand {
    pub reason: FileSyncType,
    pub name: String,
    pub text: Option<String>,
}

#[derive(Debug, Display, Error)]
pub enum FileSyncError {
    #[display(fmt = "Sync unsucessful: internal error {}", cause)]
    InternalError { cause: String },

    #[display(fmt = "Sync unsucessful: {}", cause)]
    BadClientData { cause: String },

    #[display(fmt = "Sync unsucessful: timeout")]
    Timeout,

    #[display(fmt = "Sync unsucessful: file not found")]
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
