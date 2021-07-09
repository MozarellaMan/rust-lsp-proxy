use crate::{
    config::{self, get_ls_args},
    file_system::file_sync_command::FileSyncError,
    AppState,
};
use actix_web::{web, HttpRequest, HttpResponse, Result};
use std::path::{Path, PathBuf};

use super::{runners::run_java_prog, user_program::UserProgramError};

/// Starts a websocket to run the requested file
pub async fn run_program_file(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let path: PathBuf =
        req.match_info()
            .query("filename")
            .parse()
            .map_err(|_| FileSyncError::BadClientData {
                cause: "Error parsing request URL".to_string(),
            })?;

    let file_path = Path::new(&get_ls_args().codebase_path).join(path.clone());
    if !file_path.exists() {
        return Ok(HttpResponse::NotFound().body("Nothing to execute."));
    }

    let output = match &state.lang {
        config::Lang::Java => run_java_prog(req, stream, state, file_path, path).await?,
        config::Lang::C => return Err(UserProgramError::UnsupportedLanguage.into()),
        config::Lang::Custom => return Err(UserProgramError::UnsupportedLanguage.into()),
    };

    Ok(output)
}
