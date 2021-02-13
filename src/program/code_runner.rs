use actix_web::{web, HttpRequest, HttpResponse, Result};
use std::path::{Path, PathBuf};

use tokio::stream::StreamExt;

use crate::{config, file_system::file_sync_msg::FileSyncError, get_ls_args, AppState};

use super::{runners::run_java_prog, user_program::UserProgramError};

const MAX_INPUT_SIZE: usize = 262_144; // max payload size is 256k

pub async fn add_program_input(
    mut payload: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FileSyncError> {
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        let chunk = chunk.map_err(|_| FileSyncError::InternalError {
            cause: "could not read input".to_string(),
        })?;
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_INPUT_SIZE {
            return Err(FileSyncError::BadClientData {
                cause: "overflow".to_string(),
            });
        }
        body.extend_from_slice(&chunk);
    }

    if let Ok(inputs) = &mut state.program_input.try_lock() {
        let input: Vec<String> = String::from_utf8_lossy(&body.to_vec())
            .split('\n')
            .map(|s| s.to_string())
            .collect();
        inputs.clear();
        inputs.extend(input);
        Ok(HttpResponse::Ok().body(format!("{:?}", inputs)))
    } else {
        Err(FileSyncError::InternalError {
            cause: "could not update program".to_string(),
        })
    }
}

pub async fn run_current_program(req: HttpRequest, state: web::Data<AppState>) -> Result<HttpResponse> {
    let path: PathBuf =
        req.match_info()
            .query("filename")
            .parse()
            .map_err(|_| FileSyncError::InternalError {
                cause: "Error parsing request URL".to_string(),
            })?;

    let file_path = Path::new(&get_ls_args().codebase_path).join(path.clone());
    if !file_path.exists() {
        return Ok(HttpResponse::NotFound().body("Nothing to execute."));
    }

    match &state.lang {
        config::Lang::Java => {
            run_java_prog(state, file_path, path).await?;
        }
        config::Lang::C => return Err(UserProgramError::UnsupportedLanguage.into()),
    }
    Ok(HttpResponse::NotFound().body("Nothing to execute."))
}

pub async fn kill_current_program() {}
