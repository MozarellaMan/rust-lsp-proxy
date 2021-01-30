use actix_files::NamedFile;
use actix_web::{http::ContentEncoding, web, HttpRequest, HttpResponse, Result};
use std::path::{Path, PathBuf};
use tokio::{fs::OpenOptions, io::AsyncWriteExt, process::Command};

use crate::{
    config,
    file_sync::{map_io_err, FileSyncError, FileSyncMsg, FileSyncType},
    get_ls_args, AppState,
};

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
            cause: "failed to run user code".to_string(),
        })?;

        let runner = Command::new("java")
            .current_dir(&state.workspace_dir)
            .arg(path)
            .output();

        let exec_output = runner.await.map_err(|_| FileSyncError::InternalError {
            cause: "failed to compile user code".to_string(),
        })?;

        let output: Vec<u8> = comp_output
            .stderr
            .into_iter()
            .chain(comp_output.stdout.into_iter())
            .chain(exec_output.stderr.into_iter())
            .chain(exec_output.stdout.into_iter())
            .collect();

        return Ok(HttpResponse::Ok().body(output));
    }

    Ok(HttpResponse::NotFound().body("Nothing to execute."))
}

pub async fn get_file(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse()?;
    let path = Path::new(&get_ls_args().codebase_path).join(path);
    let file = NamedFile::open(path)?
        .set_content_type(mime::TEXT_PLAIN_UTF_8)
        .set_content_encoding(ContentEncoding::Gzip);
    Ok(file)
}

pub async fn update_file(
    req: HttpRequest,
    update: web::Json<FileSyncMsg>,
) -> Result<HttpResponse, FileSyncError> {
    let path: PathBuf =
        req.match_info()
            .query("filename")
            .parse()
            .map_err(|_| FileSyncError::InternalError {
                cause: "Error parsing request URL".to_string(),
            })?;
    let path = Path::new(&get_ls_args().codebase_path).join(path);
    match update.reason {
        FileSyncType::New => {
            if path.is_dir() {
                let path = path.join(&update.name);
                let _file = tokio::fs::File::create(&path).await.map_err(map_io_err)?;
                Ok(HttpResponse::Ok().body(&path.display().to_string()))
            } else {
                Err(FileSyncError::BadClientData {
                    cause: "Cannot create new file in non-directory.".to_string(),
                })
            }
        }
        FileSyncType::Update => {
            let mut options = OpenOptions::new();
            let mut file = options
                .write(true)
                .truncate(true)
                .open(path)
                .await
                .map_err(map_io_err)?;
            file.write_all(update.text.as_bytes())
                .await
                .map_err(map_io_err)?;
            Ok(HttpResponse::Ok().body("Sync Successful"))
        }
        FileSyncType::Delete => Err(FileSyncError::InternalError {
            cause: "File deletion not implemented.".to_string(),
        }),
    }
}
