use actix_web::{dev::HttpResponseBuilder, error, http::header, http::StatusCode, HttpResponse};
use derive_more::{Display, Error};
use serde::Deserialize;
use std::path::PathBuf;
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

#[derive(Deserialize, Debug)]
pub enum FileSyncType {
    New,
    Update,
    Delete,
}
#[derive(Deserialize, Debug)]
pub struct FileSyncMsg {
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

pub async fn update_file(path: PathBuf, update: FileSyncMsg) -> Result<(), FileSyncError> {
    match update.reason {
        FileSyncType::New => {
            if path.is_dir() {
                let path = path.join(&update.name);
                let _file = tokio::fs::File::create(&path).await.map_err(map_io_err)?;
            } else {
                return Err(FileSyncError::BadClientData {
                    cause: "Cannot create new file in non-directory.".to_string(),
                });
            }
        }
        FileSyncType::Update => {
            if update.text.is_some() {
                let mut options = OpenOptions::new();
                let mut file = options
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .await
                    .map_err(map_io_err)?;
                file.write_all(update.text.unwrap().as_bytes())
                    .await
                    .map_err(map_io_err)?;
            }
        }
        FileSyncType::Delete => {
            return Err(FileSyncError::InternalError {
                cause: "File deletion not implemented.".to_string(),
            });
        }
    }
    Ok(())
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
