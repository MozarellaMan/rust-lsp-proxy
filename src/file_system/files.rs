use serde::{Deserialize, Serialize};
use walkdir::DirEntry;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
    pub path: String,
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: String,
    pub children: Vec<FileNode>,
}

impl FileNode {
    pub fn new(entry: &DirEntry) -> FileNode {
        FileNode {
            path: entry.path().display().to_string(),
            name: String::from(entry.file_name().to_str().unwrap()),
            file_type: if entry.file_type().is_dir() {
                "directory".to_owned()
            } else {
                entry
                    .path()
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned()
            },
            children: Vec::<FileNode>::new(),
        }
    }

    fn find_child(&mut self, name: String) -> Option<&mut FileNode> {
        for c in self.children.iter_mut() {
            if c.name == name {
                return Some(c);
            }
        }
        None
    }

    fn add_child<T>(&mut self, leaf: T) -> &mut Self
    where
        T: Into<FileNode>,
    {
        self.children.push(leaf.into());
        self
    }
}

pub fn build_file_tree(node: &mut FileNode, parts: &[DirEntry], depth: usize) {
    if depth < parts.len() {
        let item = &parts[depth];

        let mut dir = match node.find_child(item.file_name().to_str().unwrap().to_string()) {
            Some(d) => d,
            None => {
                let d = FileNode::new(&item);
                node.add_child(d);
                match node.find_child(item.file_name().to_str().unwrap().to_string()) {
                    Some(d2) => d2,
                    None => panic!("Got here!"),
                }
            }
        };
        build_file_tree(&mut dir, parts, depth + 1);
    }
}

pub fn is_ignored(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.') || s.starts_with("jdt") || s.ends_with(".class"))
        .unwrap_or(false)
}
