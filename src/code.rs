use actix_files::NamedFile;
use actix_web::{http::ContentEncoding, HttpRequest, Result};
use std::path::{Path, PathBuf};

use crate::get_ls_args;

pub async fn get_file(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let path = Path::new(&get_ls_args().codebase_path).join(path);
    let file = NamedFile::open(path)?
        .set_content_type(mime::TEXT_PLAIN_UTF_8)
        .set_content_encoding(ContentEncoding::Gzip);
    Ok(file)
}
