use crate::{Directory, Document};
use serde::Serialize;

use std::ffi::OsStr;
use std::path::PathBuf;

impl From<&Directory> for Level {
    fn from(dir: &Directory) -> Level {
        let index = dir
            .docs
            .iter()
            .find(|d| d.original_file_name() == Some(OsStr::new("README.md")))
            .expect("No index file found for directory");

        let links = dir
            .docs
            .iter()
            .filter(|d| *d != index)
            .map(|d| d.into())
            .collect();
        let children = dir.dirs.iter().map(|d| d.into()).collect();

        Level {
            index: index.into(),
            links,
            children,
        }
    }
}

impl From<&Document> for Link {
    fn from(doc: &Document) -> Link {
        Link {
            path: PathBuf::from("/").join(doc.relative_path()),
            title: doc.title().to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Level {
    index: Link,
    links: Vec<Link>,
    children: Vec<Level>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Link {
    pub path: PathBuf,
    pub title: String,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;

    fn page(path: &str, name: &str) -> Document {
        let mut frontmatter = BTreeMap::new();
        frontmatter.insert("title".to_string(), name.to_string());

        Document::new(path, "docs/", "Not important".to_string(), frontmatter)
    }

    #[test]
    fn basic() {
        let root = Directory {
            docs: vec![
                page("docs/README.md", "Getting Started"),
                page("docs/one.md", "One"),
                page("docs/two.md", "Two"),
            ],
            dirs: vec![Directory {
                docs: vec![
                    page("docs/child/README.md", "Nested Root"),
                    page("docs/child/three.md", "Three"),
                ],
                dirs: vec![],
            }],
        };

        assert_eq!(
            Level::from(&root),
            Level {
                index: Link {
                    path: PathBuf::from("/index.html"),
                    title: "Getting Started".to_string(),
                },
                links: vec![
                    Link {
                        path: PathBuf::from("/one.html"),
                        title: "One".to_string()
                    },
                    Link {
                        path: PathBuf::from("/two.html"),
                        title: "Two".to_string(),
                    }
                ],
                children: vec![Level {
                    index: Link {
                        path: PathBuf::from("/child/index.html"),
                        title: "Nested Root".to_string()
                    },
                    links: vec![Link {
                        path: PathBuf::from("/child/three.html"),
                        title: "Three".to_string()
                    },],
                    children: vec![]
                }]
            }
        )
    }
}
