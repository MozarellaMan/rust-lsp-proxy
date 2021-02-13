use crate::{
    file_sync_msg::{map_io_err, FileSyncError, FileSyncMsg, FileSyncType},
    files::{build_file_tree, is_ignored, FileNode},
    get_ls_args, AppState,
};
use actix_files::NamedFile;
use actix_web::{
    http::ContentEncoding,
    web::{self, Json},
    HttpRequest, HttpResponse, Responder, Result,
};
use std::path::{Path, PathBuf};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use walkdir::{DirEntry, WalkDir};

pub async fn get_root_uri(state: web::Data<AppState>) -> impl Responder {
    let uri = format!("file:///{}", &state.workspace_dir);
    HttpResponse::Ok().body(uri)
}

pub async fn get_file(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse()?;
    let path = Path::new(&get_ls_args().codebase_path).join(path);
    let file = NamedFile::open(path)?
        .set_content_type(mime::TEXT_PLAIN_UTF_8)
        .set_content_encoding(ContentEncoding::Gzip);
    Ok(file)
}

pub async fn get_dir() -> Result<Json<FileNode>, std::io::Error> {
    let mut paths: Vec<DirEntry> = Vec::new();
    for entry in WalkDir::new(get_ls_args().codebase_path)
        .into_iter()
        .filter_entry(|e| !is_ignored(&e))
        .filter_map(|e| e.ok())
    {
        paths.push(entry);
    }
    let mut top = FileNode::new(paths.get(0).unwrap());
    for _path in paths.iter() {
        build_file_tree(&mut top, &paths, 1);
    }
    Ok(Json(top))
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
