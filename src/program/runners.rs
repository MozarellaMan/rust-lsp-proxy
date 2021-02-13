use std::{path::PathBuf, process::Stdio};

use actix_web::{web, HttpResponse, Result};
use tokio::process::Command;

use crate::AppState;

use super::user_program::{UserProgram, UserProgramError};

pub async fn run_java_prog(
    state: web::Data<AppState>,
    file_path: PathBuf,
    path: PathBuf,
) -> Result<HttpResponse> {
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
                .map_err(|_| UserProgramError::FailedCompilation)?,
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
    Err(UserProgramError::FailedRun.into())
}
