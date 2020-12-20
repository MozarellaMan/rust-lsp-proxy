use actix_web::{
    dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
enum FileSyncError {
    #[display(fmt = "internal error {}", cause)]
    InternalError { cause: String },

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "timeout")]
    Timeout,
}

impl error::ResponseError for FileSyncError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            FileSyncError::InternalError{..} => StatusCode::INTERNAL_SERVER_ERROR,
            FileSyncError::BadClientData => StatusCode::BAD_REQUEST,
            FileSyncError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}