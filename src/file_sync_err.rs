use actix_web::{
    dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse,
};
use derive_more::{Display, Error};

#[derive(Debug, Display)]
enum ErrType {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "timeout")]
    Timeout,
}
#[derive(Debug, Display, Error)]
#[display(fmt = "{}: {}", _type, message)]
pub struct FileSyncError {
    _type: ErrType,
    message: &'static str,

}

impl error::ResponseError for FileSyncError {
    fn error_response(&self) -> HttpResponse {
        match self._type {
            ErrType::InternalError => {
                HttpResponseBuilder::new(self.status_code())
                    .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                    .body(self.message)
            }
            _ => {
                HttpResponseBuilder::new(self.status_code())
                .set_header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(self.to_string())
            }
        }
        
    }

    fn status_code(&self) -> StatusCode {
        match self._type {
            ErrType::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            ErrType::BadClientData => StatusCode::BAD_REQUEST,
            ErrType::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}