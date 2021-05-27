use super::{
    file_sync_msg::{map_io_err, FileSyncError, FileSyncMsg, FileSyncType},
    files::{build_file_tree, FileNode},
};
use crate::{config::get_ls_args, AppState};
use actix_files::NamedFile;
use actix_web::{
    http::ContentEncoding,
    web::{self, Json},
    HttpRequest, HttpResponse, Responder, Result,
};
use std::path::{Path, PathBuf};
use tokio::{fs::OpenOptions, io::AsyncWriteExt};

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
    let dir = build_file_tree(&get_ls_args().codebase_path, 0);
    Ok(Json(dir))
}

pub async fn update_file(path: PathBuf, update: FileSyncMsg) -> Result<(), FileSyncError> {
    match update.reason {
        FileSyncType::New => {
            if path.is_dir() {
                println!("creating!");
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

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::file_system::file_sync_msg::{FileSyncMsg, FileSyncType};

    use super::update_file;
    use tempfile::{self, tempdir, NamedTempFile};

    #[actix_rt::test]
    async fn check_update_file_works() {
        let initial_contents = "Hello World!";
        let expected_contents = "Hello World!!! \n How do?";
        let mut file = NamedTempFile::new().expect("couldn't create file for testing!");
        file.write_all(initial_contents.as_bytes())
            .expect("could not write to test file!");

        update_file(
            file.path().to_path_buf(),
            FileSyncMsg {
                reason: FileSyncType::Update,
                name: file
                    .path()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                text: Some(expected_contents.to_string()),
            },
        )
        .await
        .expect("Error in updating file!");

        let actual_contents = std::fs::read_to_string(file).expect("could not read test file!");

        assert_eq!(actual_contents, expected_contents);
        assert_eq!(expected_contents.len(), actual_contents.len());
    }

    #[actix_rt::test]
    async fn check_create_file_works() {
        let new_file_content = "Hello World!";
        let new_file_name = "Newfile.txt";
        let dir = tempdir().expect("couldn't create directory for testing!");

        update_file(
            dir.path().to_path_buf(),
            FileSyncMsg {
                reason: FileSyncType::New,
                name: new_file_name.to_string(),
                text: Some(new_file_content.to_string()),
            },
        )
        .await
        .expect("Error in creating file!");

        let new_file_path = dir.path().join(new_file_name);
        let actual_contents =
            std::fs::read_to_string(new_file_path).expect("could not read test file!");

        assert_eq!(actual_contents, new_file_content);
    }
}
