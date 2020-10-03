use crate::config::{Config, Navigation as NavRule};
use crate::{Directory, Document, Error, Result};
use serde::Serialize;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub struct Navigation<'a> {
    config: &'a Config,
}

impl<'a> Navigation<'a> {
    pub fn new(config: &'a Config) -> Self {
        Navigation { config }
    }

    pub fn build_for(&self, dir: &Directory) -> Level {
        let default = Level::from(dir);

        match &self.config.navigation() {
            None => default,
            Some(_nav) => self.customize(default),
        }
    }

    fn customize(&self, default: Level) -> Level {
        let rules = self.config.navigation().unwrap();

        let root = Level {
            index: default.index.clone(),
            links: vec![],
            children: vec![],
        };

        // for rule in rules {
        //     if self.config.project_root().join(rule.path).is_file() {
        //     }
        // }

        root
    }
}

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
        let mut tmp = doc.html_path().clone();

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
    use std::path::Path;

    fn page(path: &str, name: &str) -> Document {
        let mut frontmatter = BTreeMap::new();
        frontmatter.insert("title".to_string(), name.to_string());

        Document::new(Path::new(path), "Not important".to_string(), frontmatter)
    }

    fn config(yaml: Option<&str>) -> Config {
        let conf = yaml.unwrap_or("---\ntitle: My project\n");

        Config::from_yaml_str(&Path::new("project"), conf).unwrap()
    }

    #[test]
    fn basic() {
        let config = config(None);
        let root = Directory {
            path: PathBuf::from("docs"),
            docs: vec![
                page("README.md", "Getting Started"),
                page("one.md", "One"),
                page("two.md", "Two"),
            ],
            dirs: vec![Directory {
                path: PathBuf::from("docs").join("child"),
                docs: vec![
                    page("child/README.md", "Nested Root"),
                    page("child/three.md", "Three"),
                ],
                dirs: vec![],
            }],
        };

        let navigation = Navigation::new(&config);

        assert_eq!(
            navigation.build_for(&root),
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
        let config = config(None);
        let root = Directory {
            path: PathBuf::from("docs"),
            docs: vec![
                page("README.md", "Getting Started"),
                page("001.md", "bb"),
                page("002.md", "11"),
            ],
            dirs: vec![
                Directory {
                    path: PathBuf::from("docs").join("bb_child"),
                    docs: vec![
                        page("child/README.md", "Index"),
                        page("child/001.md", "BB"),
                        page("child/002.md", "22"),
                        page("child/003.md", "AA"),
                        page("child/004.md", "11"),
                    ],
                    dirs: vec![],
                },
                Directory {
                    path: PathBuf::from("docs").join("aa_child"),
                    docs: vec![
                        page("child2/README.md", "Index"),
                        page("child2/001.md", "123"),
                        page("child2/002.md", "aa"),
                        page("child2/003.md", "cc"),
                        page("child2/004.md", "bb"),
                    ],
                    dirs: vec![],
                },
            ],
        };

        let navigation = Navigation::new(&config);

        assert_eq!(
            navigation.build_for(&root),
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

    #[test]
    fn manual_menu_simple() {
        let root = Directory {
            path: PathBuf::from("docs"),
            docs: vec![
                page("README.md", "Getting Started"),
                page("one.md", "One"),
                page("two.md", "Two"),
            ],
            dirs: vec![Directory {
                path: PathBuf::from("docs").join("child"),
                docs: vec![
                    page("child/README.md", "Nested Root"),
                    page("child/three.md", "Three"),
                ],
                dirs: vec![],
            }],
        };

        let config = config(Some(&indoc! {"
            ---
            title: My project
            navigation:
              - path: docs/one.md
              - path: docs/child/README.md
                children: \"*\"
            "}));
        let navigation = Navigation::new(&config);

        assert_eq!(
            navigation.build_for(&root),
            Level {
                index: Link {
                    path: String::from("/"),
                    title: String::from("Getting Started"),
                },
                links: vec![Link {
                    path: String::from("/one"),
                    title: String::from("One")
                },],
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
