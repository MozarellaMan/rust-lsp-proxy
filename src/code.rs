use actix_files::NamedFile;
use actix_web::{http::ContentEncoding, web, HttpRequest, HttpResponse, Result};
use std::{
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{io::AsyncWriteExt, process::Command};
use tokio::{stream::StreamExt, time::timeout};

use crate::{config, file_sync::FileSyncError, get_ls_args, AppState};

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

    if let Ok(inputs) = &mut state.code_input.try_lock() {
        //let input = body.to_owned()
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

pub async fn run_file(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> Result<HttpResponse, FileSyncError> {
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

        let comp_output = compiler.await.map_err(|_| FileSyncError::InternalError {
            cause: "failed to compile user code".to_string(),
        })?;

        let mut runner = Command::new("java")
            .current_dir(&state.workspace_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .arg(path)
            .spawn()
            .map_err(|_| FileSyncError::InternalError {
                cause: "failed to spawn user code".to_string(),
            })?;
        {
            read_user_program_input(state, &mut runner).await;
        }

        let run_output = timeout(Duration::from_secs(300), runner.wait_with_output())
            .await
            .map_err(|_| FileSyncError::InternalError {
                cause: "failed to run user code, timed out after 5 minues".to_string(),
            })?
            .map_err(|_| FileSyncError::InternalError {
                cause: "failed to run user code".to_string(),
            })?;

        let output: Vec<u8> = comp_output
            .stderr
            .into_iter()
            .chain(comp_output.stdout.into_iter())
            .chain(run_output.stdout.into_iter())
            .chain(run_output.stderr.into_iter())
            .collect();

        return Ok(HttpResponse::Ok().body(output));
    }

    Ok(HttpResponse::NotFound().body("Nothing to execute."))
}

async fn read_user_program_input(state: web::Data<AppState>, runner: &mut tokio::process::Child) {
    if let Ok(inputs) = state.code_input.try_lock() {
        if !inputs.is_empty() {
            if let Some(stdin) = &mut runner.stdin {
                for input in inputs.iter() {
                    if let Err(er) = stdin.write_all(&input.as_bytes()).await {
                        eprintln!("Error writing to child process {:?}", er);
                    }
                    stdin.flush();
                }
            }
        }
    }
}

pub async fn get_file(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse()?;
    let path = Path::new(&get_ls_args().codebase_path).join(path);
    let file = NamedFile::open(path)?
        .set_content_type(mime::TEXT_PLAIN_UTF_8)
        .set_content_encoding(ContentEncoding::Gzip);
    Ok(file)
}
