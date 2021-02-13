use std::path::PathBuf;
use actix_web::{HttpResponse, Responder, web::{self, Json}};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};
use walkdir::{DirEntry, WalkDir};
use crate::{AppState, file_sync_msg::{FileSyncError, FileSyncMsg, FileSyncType, map_io_err}, files::{FileNode, build_file_tree, is_ignored}, get_ls_args};

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

pub async fn get_root_uri(state: web::Data<AppState>) -> impl Responder {
    let uri = format!("file:///{}", &state.workspace_dir);
    HttpResponse::Ok().body(uri)
}

