use std::{path::PathBuf, process::Stdio};

use actix_web::{web, HttpRequest, HttpResponse, Result};
use actix_web_actors::ws;
use tokio::process::Command;

use crate::AppState;

use super::user_program::{UserProgram, UserProgramError};

pub async fn run_java_prog(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
    file_path: PathBuf,
    path: PathBuf,
) -> Result<HttpResponse> {
    let compiler = Command::new("javac")
        .current_dir(&state.workspace_dir)
        .arg(&file_path)
        .output();

    compiler
        .await
        .map_err(|_| UserProgramError::FailedCompilation)?;

    if let Ok(user_program) = &mut state.user_program.try_lock() {
        user_program.replace(UserProgram::start(
            Command::new("java")
                .kill_on_drop(true)
                .current_dir(&state.workspace_dir)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .arg(path)
                .spawn()
                .map_err(|_| UserProgramError::FailedRun)?,
        ));

        if user_program.is_some() {
            let user_program = ws::start(user_program.take().unwrap(), &req, stream)
                .map_err(|_| UserProgramError::FailedRun)?;
            return Ok(user_program);
        }
    } else {
        return Err(UserProgramError::FailedProgramLock.into());
    }
    Err(UserProgramError::FailedRun.into())
}
