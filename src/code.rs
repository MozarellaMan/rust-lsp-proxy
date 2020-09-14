use actix_files::NamedFile;
use actix_web::{
    http::header::ContentDisposition, http::header::DispositionParam,
    http::header::DispositionType, http::ContentEncoding, HttpRequest, Result,
};
use mime;
use std::path::{Path, PathBuf};

pub async fn get_file(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let path = Path::new("./tests/example_code_repos/test-java-repo").join(path);
    let file = NamedFile::open(path)?
        .set_content_type(mime::TEXT_PLAIN_UTF_8)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(String::from("Hello.java"))],
        })
        .set_content_encoding(ContentEncoding::Gzip);
    Ok(file)
}
