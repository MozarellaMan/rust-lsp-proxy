use std::path::Path;

use lsp_types::{CreateFilesParams, DidChangeTextDocumentParams, Url};
use serde_json::Value;

use crate::file_system::{
    file_sync::update_file,
    file_sync_msg::{FileSyncMsg, FileSyncType},
};

type SerializerError = serde_json::error::Error;

pub async fn intercept_notification(msg: Value) -> Result<(), SerializerError> {
    if let Value::String(method) = &msg["method"] {
        if method.starts_with("textDocument/didChange") || method.starts_with("workspace/didCreate")
        {
            intercept_text_sync(&msg, method).await?;
        }
    } else {
        println!("not method! : {:?} ", &msg);
    }
    Ok(())
}

async fn intercept_text_sync(msg: &Value, method: &str) -> Result<(), SerializerError> {
    if let Value::Object(_) = &msg["params"] {
        match method {
            "textDocument/didChange" => {
                let did_update: DidChangeTextDocumentParams =
                    serde_json::from_value(msg["params"].clone())?;
                intercept_did_update(did_update).await;
            }
            "workspace/didCreateFiles" => {
                let did_create: CreateFilesParams = serde_json::from_value(msg["params"].clone())?;
                intercept_did_create(did_create).await;
            }
            _unrecog => {
                println!("not recongized: {}", _unrecog);
            }
        }
    }
    Ok(())
}

async fn intercept_did_create(params: CreateFilesParams) {
    let file_creates = params.files;
    for creation in file_creates.iter() {
        let uri = Url::parse(&creation.uri);
        if let Ok(url) = uri {
            let mut path = Path::new(url.as_str()).to_path_buf();
            let file_name = url.path_segments().map(|s| s.last()).unwrap_or_default();
            if let Some(name) = file_name {
                let file_sync_msg = FileSyncMsg {
                    reason: FileSyncType::New,
                    name: name.to_string(),
                    text: None,
                };
                path.pop();
                if let Err(err) = update_file(path.clone(), file_sync_msg).await {
                    println!("could not update! {}", err);
                } else {
                    println!("Created file: {:?}{:?}", path, file_name);
                }
            } else {
                println!("could not create path:{:?},{:?}", path, file_name);
            }
        }
    }
}

async fn intercept_did_update(params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri;
    let path = uri.to_file_path();
    let file_name = uri.path_segments().map(|s| s.last()).unwrap_or_default();
    if let (Ok(path), Some(name)) = (path, file_name) {
        for change in params.content_changes.iter() {
            let file_sync_msg = FileSyncMsg {
                reason: FileSyncType::Update,
                name: name.to_string(),
                text: Some(change.text.clone()),
            };
            if let Err(err) = update_file(path.clone(), file_sync_msg).await {
                println!("{}", err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn intercept_detects_correct_method() {
        let _methods = vec!["textDocument/didUpdate", "workspace/didCreateFiles"];
    }
}
