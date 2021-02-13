use actix_web::{web, HttpRequest, HttpResponse, Result};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::process::Command;
use tokio::stream::StreamExt;

use crate::{config, file_system::file_sync_msg::FileSyncError, get_ls_args, AppState};

use super::user_program::{UserProgram, UserProgramError};

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

pub async fn run_file(req: HttpRequest, state: web::Data<AppState>) -> Result<HttpResponse> {
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

    if let config::Lang::Java = &state.lang {
        let compiler = Command::new("javac")
            .current_dir(&state.workspace_dir)
            .arg(&file_path)
            .output();

        let comp_output = compiler
            .await
            .map_err(|_| UserProgramError::FailedCompilation)?;

        if let Ok(user_program) = &mut state.user_program.try_lock() {
            user_program.replace(UserProgram(Some(
                Command::new("java")
                    .current_dir(&state.workspace_dir)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .arg(path)
                    .spawn()
                    .map_err(|_| FileSyncError::InternalError {
                        cause: "failed to spawn user code".to_string(),
                    })?,
            )));

            if user_program.is_some() {
                let user_program = user_program.as_mut().unwrap();

                if let Ok(inputs) = state.program_input.try_lock() {
                    user_program.read_user_program_input(&inputs).await?
                }

                let run_output = user_program.wait_with_output().await?;

                let output: Vec<u8> = comp_output
                    .stderr
                    .into_iter()
                    .chain(comp_output.stdout.into_iter())
                    .chain(run_output)
                    .collect();

                return Ok(HttpResponse::Ok().body(output));
            }
        }
    }
    Ok(HttpResponse::NotFound().body("Nothing to execute."))
}
