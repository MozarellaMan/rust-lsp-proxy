use super::{
    file_sync_command::{map_io_err, FileSyncCommand, FileSyncError, FileSyncType},
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

/// Handles file synchronization commands. Makes the actual changes to the file system
pub async fn handle_file_sync(
    path: PathBuf,
    command: FileSyncCommand,
) -> Result<(), FileSyncError> {
    match command.reason {
        FileSyncType::New => {
            if path.is_dir() {
                let path = path.join(&command.name);
                let _file = tokio::fs::File::create(&path).await.map_err(map_io_err)?;
            } else {
                return Err(FileSyncError::BadClientData {
                    cause: "Cannot create new file in non-directory.".to_string(),
                });
            }
        }
        FileSyncType::Update => {
            if command.text.is_some() {
                let mut options = OpenOptions::new();
                let mut file = options
                    .write(true)
                    .truncate(true)
                    .open(path)
                    .await
                    .map_err(map_io_err)?;
                file.write_all(command.text.unwrap().as_bytes())
                    .await
                    .map_err(map_io_err)?;
            }
        }
        FileSyncType::Delete => {
            if path.exists() {
                tokio::fs::remove_file(path).await.map_err(map_io_err)?
            } else {
                return Err(FileSyncError::NotFound);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::file_system::file_sync_command::{FileSyncCommand, FileSyncType};

    use super::handle_file_sync;
    use tempfile::{self, tempdir, NamedTempFile};

    #[actix_rt::test]
    async fn check_update_file_works() {
        let initial_contents = "Hello World!";
        let expected_contents = "Hello World!!! \n How do?";
        let mut file = NamedTempFile::new().expect("couldn't create file for testing!");
        file.write_all(initial_contents.as_bytes())
            .expect("could not write to test file!");

        handle_file_sync(
            file.path().to_path_buf(),
            FileSyncCommand {
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

        handle_file_sync(
            dir.path().to_path_buf(),
            FileSyncCommand {
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
