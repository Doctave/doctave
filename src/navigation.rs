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

        let mut links = dir
            .docs
            .iter()
            .filter(|d| *d != index)
            .map(|d| d.into())
            .collect::<Vec<Link>>();

        let mut children = dir.dirs.iter().map(|d| d.into()).collect::<Vec<Level>>();

        links.sort_by(|a, b| a.title.cmp(&b.title));
        children.sort_by(|a, b| a.index.title.cmp(&b.index.title));

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
            tmp = tmp
                .parent()
                .map(|p| p.to_owned())
                .unwrap_or(PathBuf::from(""));
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
            path: PathBuf::from("docs"),
            docs: vec![
                page("docs/README.md", "Getting Started"),
                page("docs/one.md", "One"),
                page("docs/two.md", "Two"),
            ],
            dirs: vec![Directory {
                path: PathBuf::from("docs").join("child"),
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

    #[test]
    fn sorting_alphanumerically() {
        let root = Directory {
            path: PathBuf::from("docs"),
            docs: vec![
                page("docs/README.md", "Getting Started"),
                page("docs/001.md", "bb"),
                page("docs/002.md", "11"),
            ],
            dirs: vec![
                Directory {
                    path: PathBuf::from("docs").join("bb_child"),
                    docs: vec![
                        page("docs/child/README.md", "Index"),
                        page("docs/child/001.md", "BB"),
                        page("docs/child/002.md", "22"),
                        page("docs/child/003.md", "AA"),
                        page("docs/child/004.md", "11"),
                    ],
                    dirs: vec![],
                },
                Directory {
                    path: PathBuf::from("docs").join("aa_child"),
                    docs: vec![
                        page("docs/child2/README.md", "Index"),
                        page("docs/child2/001.md", "123"),
                        page("docs/child2/002.md", "aa"),
                        page("docs/child2/003.md", "cc"),
                        page("docs/child2/004.md", "bb"),
                    ],
                    dirs: vec![],
                },
            ],
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
                        path: String::from("/002"),
                        title: String::from("11")
                    },
                    Link {
                        path: String::from("/001"),
                        title: String::from("bb"),
                    }
                ],
                children: vec![
                    Level {
                        index: Link {
                            path: String::from("/child"),
                            title: String::from("Index")
                        },
                        links: vec![
                            Link {
                                path: String::from("/child/004"),
                                title: String::from("11")
                            },
                            Link {
                                path: String::from("/child/002"),
                                title: String::from("22")
                            },
                            Link {
                                path: String::from("/child/003"),
                                title: String::from("AA")
                            },
                            Link {
                                path: String::from("/child/001"),
                                title: String::from("BB")
                            },
                        ],
                        children: vec![]
                    },
                    Level {
                        index: Link {
                            path: String::from("/child2"),
                            title: String::from("Index")
                        },
                        links: vec![
                            Link {
                                path: String::from("/child2/001"),
                                title: String::from("123")
                            },
                            Link {
                                path: String::from("/child2/002"),
                                title: String::from("aa")
                            },
                            Link {
                                path: String::from("/child2/004"),
                                title: String::from("bb")
                            },
                            Link {
                                path: String::from("/child2/003"),
                                title: String::from("cc")
                            },
                        ],
                        children: vec![]
                    }
                ]
            }
        )
    }
}
