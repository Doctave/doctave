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
        let mut tmp = doc.relative_path().clone();

        // Default to stipping .html extensions
        tmp.set_extension("");

        if tmp.file_name() == Some(OsStr::new("index")) {
            tmp = tmp.parent().map(|p| p.to_owned()).unwrap_or(PathBuf::from(""));
        }

        // Need to force forward slashes here, since URIs will always
        // work the same across all platforms.
        let uri_path = tmp
            .components()
            .into_iter()
            .map(|c| format!("{}", c.as_os_str().to_string_lossy()))
            .collect::<Vec<_>>()
            .join("/");

        Link {
            path: format!("/{}", uri_path),
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
    pub path: String,
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
                    path: String::from("/"),
                    title: String::from("Getting Started"),
                },
                links: vec![
                    Link {
                        path: String::from("/one"),
                        title: String::from("One")
                    },
                    Link {
                        path: String::from("/two"),
                        title: String::from("Two"),
                    }
                ],
                children: vec![Level {
                    index: Link {
                        path: String::from("/child"),
                        title: String::from("Nested Root")
                    },
                    links: vec![Link {
                        path: String::from("/child/three"),
                        title: String::from("Three")
                    },],
                    children: vec![]
                }]
            }
        )
    }
}
