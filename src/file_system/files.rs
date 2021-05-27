use std::path::Path;

use path_slash::PathExt;
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileNode {
    pub path: String,
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub children: Vec<FileNode>,
}

impl FileNode {
    pub fn new(entry: &Path) -> FileNode {
        let name = entry.file_name().unwrap().to_string_lossy().to_string();
        FileNode {
            path: entry.to_slash_lossy(),
            file_type: if entry.is_dir() {
                "directory".to_owned()
            } else {
                Path::new(&name)
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap()
                    .to_string()
            },
            name,
            children: Vec::<FileNode>::new(),
        }
    }

    fn add_child<T>(&mut self, leaf: T) -> &mut Self
    where
        T: Into<FileNode>,
    {
        self.children.push(leaf.into());
        self
    }
}

pub fn build_file_tree(path: &str, depth: usize) -> FileNode {
    let mut root_node = FileNode::new(Path::new(path));

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| !is_ignored(&e))
        .filter_map(|e| e.ok())
        .skip(1)
    {
        if let Ok(entry_metadata) = entry.metadata() {
            if entry_metadata.is_dir() && entry.depth() == depth + 1 {
                let new_dir = build_file_tree(
                    entry
                        .path()
                        .to_str()
                        .expect("Path to string conversion error"),
                    0,
                );
                root_node.add_child(new_dir);
            } else if entry.depth() == depth + 1 {
                root_node.add_child(FileNode::new(entry.path()));
            }
        }
    }
    root_node
}

/// Ignores class files and dot directories by default
pub fn is_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') || s.ends_with(".class"))
        .unwrap_or(false)
}
