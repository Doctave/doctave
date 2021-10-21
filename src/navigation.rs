use crate::config::{Config, DirIncludeRule, NavRule};
use crate::Directory;
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

    /// Builds a navigation tree given a root directory
    pub fn build_for(&self, dir: &Directory) -> Vec<Link> {
        let default: Vec<Link> = dir.links();

        match &self.config.navigation() {
            None => default,
            Some(nav) => self.customize(nav, &default),
        }
    }

    /// Customizes the navigation tree given some rules provided through the
    /// doctave.yaml config.
    ///
    /// Note that the config validates that any files/directories referenced
    /// in the rules already exist, which is why we can reasonably confidently
    /// unwrap some Nones here. The only case they would trip is if the files
    /// got removed between the validation and building these rules, which is
    /// a _very_ small window.
    ///
    /// Note that in the case where an explicit path is provided, the link is
    /// not necessarily a direct child of its parent. It could be that links
    /// under a directory actually point to a parent's sibling, or to somewhere
    /// else in the tree.
    fn customize(&self, rules: &[NavRule], default: &[Link]) -> Vec<Link> {
        let mut links = vec![];

        for rule in rules {
            match rule {
                NavRule::File(path) => links.push(
                    self.find_matching_link(path, &default)
                        .expect("No matching link found"),
                ),
                NavRule::Dir(path, dir_rule) => {
                    let mut index_link = self
                        .find_matching_link(path, &default)
                        .expect("No matching link found");

                    match dir_rule {
                        // Don't include any children
                        None => {
                            index_link.children.truncate(0);
                            links.push(index_link);
                        }
                        // Include all children
                        Some(DirIncludeRule::WildCard) => links.push(index_link),
                        // Include only links that match the description
                        Some(DirIncludeRule::Explicit(nested_rules)) => {
                            let children = self.customize(nested_rules, &default);
                            index_link.children = children;
                            links.push(index_link);
                        }
                    }
                }
            }
        }

        links
    }

    /// Matches a path provided in a NavRule to a Link. Recursively searches through
    /// the link children to find a match.
    fn find_matching_link(&self, path: &Path, links: &[Link]) -> Option<Link> {
        let search_result = links.iter().find(|link| {
            let mut without_docs_part = path.components();
            let _ = without_docs_part.next();

            let link_path = link
                    .path
                    .strip_prefix(self.config.base_path())
                    .unwrap();

            let doc_path = Link::path_to_uri(without_docs_part.as_path());

            link_path.trim_start_matches("/") == doc_path.trim_start_matches("/")
        });

        match search_result {
            Some(link) => Some(link.clone()),
            None => {
                let recursive_results = links
                    .iter()
                    .flat_map(|l| self.find_matching_link(path, &l.children))
                    .collect::<Vec<_>>();

                // _Should_ only be one match, if any
                return recursive_results.get(0).map(|l| l.clone());
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Link {
    pub path: String,
    pub title: String,
    pub children: Vec<Link>,
}

impl Link {
    pub fn path_to_uri(path: &Path) -> String {
        let mut tmp = path.to_owned();

        // Default to stripping .html extensions
        tmp.set_extension("");

        if tmp.file_name() == Some(OsStr::new("index")) {
            tmp = tmp
                .parent()
                .map(|p| p.to_owned())
                .unwrap_or_else(|| PathBuf::from(""));
        }

        // Need to force forward slashes here, since URIs will always
        // work the same across all platforms.
        let uri_path = tmp
            .components()
            .into_iter()
            .map(|c| format!("{}", c.as_os_str().to_string_lossy()))
            .collect::<Vec<_>>()
            .join("/");

        format!("{}", uri_path.as_str().trim_start_matches("/"))
    }

    pub fn path_to_uri_with_extension(path: &Path) -> String {
        let mut tmp = path.to_owned();

        if tmp.file_name() == Some(OsStr::new("index")) {
            tmp = tmp
                .parent()
                .map(|p| p.to_owned())
                .unwrap_or_else(|| PathBuf::from(""));
        }

        // Need to force forward slashes here, since URIs will always
        // work the same across all platforms.
        let uri_path = tmp
            .components()
            .into_iter()
            .map(|c| format!("{}", c.as_os_str().to_string_lossy()))
            .collect::<Vec<_>>()
            .join("/");

        format!("{}", uri_path.as_str().trim_start_matches("/"))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::Path;

    use crate::Document;

    fn page(path: &str, name: &str, base_path: Option<&str>) -> Document {
        let mut frontmatter = BTreeMap::new();
        frontmatter.insert("title".to_string(), name.to_string());

        Document::new(
            Path::new(path),
            "Not important".to_string(),
            frontmatter,
            base_path.unwrap_or("/")
        )
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
                page("README.md", "Getting Started", None),
                page("one.md", "One", None),
                page("two.md", "Two", None),
            ],
            dirs: vec![Directory {
                path: PathBuf::from("docs").join("child"),
                docs: vec![
                    page("child/README.md", "Nested Root", None),
                    page("child/three.md", "Three", None),
                ],
                dirs: vec![],
            }],
        };

        let navigation = Navigation::new(&config);

        assert_eq!(
            navigation.build_for(&root),
            vec![
                Link {
                    path: String::from("/child"),
                    title: String::from("Nested Root"),
                    children: vec![Link {
                        path: String::from("/child/three"),
                        title: String::from("Three"),
                        children: vec![]
                    }]
                },
                Link {
                    path: String::from("/one"),
                    title: String::from("One"),
                    children: vec![]
                },
                Link {
                    path: String::from("/two"),
                    title: String::from("Two"),
                    children: vec![]
                },
            ]
        )
    }

    #[test]
    fn sorting_alphanumerically() {
        let config = config(None);
        let root = Directory {
            path: PathBuf::from("docs"),
            docs: vec![
                page("README.md", "Getting Started", None),
                page("001.md", "bb", None),
                page("002.md", "11", None),
            ],
            dirs: vec![
                Directory {
                    path: PathBuf::from("docs").join("bb_child"),
                    docs: vec![
                        page("child/README.md", "Index", None),
                        page("child/001.md", "BB", None),
                        page("child/002.md", "22", None),
                        page("child/003.md", "AA", None),
                        page("child/004.md", "11", None),
                    ],
                    dirs: vec![],
                },
                Directory {
                    path: PathBuf::from("docs").join("aa_child"),
                    docs: vec![
                        page("child2/README.md", "Index", None),
                        page("child2/001.md", "123", None),
                        page("child2/002.md", "aa", None),
                        page("child2/003.md", "cc", None),
                        page("child2/004.md", "bb", None),
                    ],
                    dirs: vec![],
                },
            ],
        };

        let navigation = Navigation::new(&config);

        assert_eq!(
            navigation.build_for(&root),
            vec![
                Link {
                    path: String::from("/002"),
                    title: String::from("11"),
                    children: vec![],
                },
                Link {
                    path: String::from("/child"),
                    title: String::from("Index"),
                    children: vec![
                        Link {
                            path: String::from("/child/004"),
                            title: String::from("11"),
                            children: vec![],
                        },
                        Link {
                            path: String::from("/child/002"),
                            title: String::from("22"),
                            children: vec![],
                        },
                        Link {
                            path: String::from("/child/003"),
                            title: String::from("AA"),
                            children: vec![],
                        },
                        Link {
                            path: String::from("/child/001"),
                            title: String::from("BB"),
                            children: vec![],
                        },
                    ]
                },
                Link {
                    path: String::from("/child2"),
                    title: String::from("Index"),
                    children: vec![
                        Link {
                            path: String::from("/child2/001"),
                            title: String::from("123"),
                            children: vec![]
                        },
                        Link {
                            path: String::from("/child2/002"),
                            title: String::from("aa"),
                            children: vec![]
                        },
                        Link {
                            path: String::from("/child2/004"),
                            title: String::from("bb"),
                            children: vec![]
                        },
                        Link {
                            path: String::from("/child2/003"),
                            title: String::from("cc"),
                            children: vec![]
                        },
                    ]
                },
                Link {
                    path: String::from("/001"),
                    title: String::from("bb"),
                    children: vec![],
                },
            ],
        )
    }

    #[test]
    fn manual_menu_simple() {
        let root = Directory {
            path: PathBuf::from("docs"),
            docs: vec![
                page("README.md", "Getting Started", None),
                page("one.md", "One", None),
                page("two.md", "Two", None),
            ],
            dirs: vec![Directory {
                path: PathBuf::from("docs").join("child"),
                docs: vec![
                    page("child/README.md", "Nested Root", None),
                    page("child/three.md", "Three", None),
                ],
                dirs: vec![],
            }],
        };

        let rules = vec![
            NavRule::File(PathBuf::from("docs/one.md")),
            NavRule::Dir(PathBuf::from("docs/child"), Some(DirIncludeRule::WildCard)),
        ];

        let config = config(None);
        let navigation = Navigation::new(&config);
        let links: Vec<Link> = root.links();

        assert_eq!(
            navigation.customize(&rules, &links),
            vec![
                Link {
                    path: String::from("/one"),
                    title: String::from("One"),
                    children: vec![],
                },
                Link {
                    path: String::from("/child"),
                    title: String::from("Nested Root"),
                    children: vec![Link {
                        path: String::from("/child/three"),
                        title: String::from("Three"),
                        children: vec![],
                    },],
                },
            ]
        )
    }

    #[test]
    fn manual_menu_nested() {
        let root = Directory {
            path: PathBuf::from("docs"),
            docs: vec![
                page("README.md", "Getting Started", None),
                page("one.md", "One", None),
                page("two.md", "Two", None),
            ],
            dirs: vec![Directory {
                path: PathBuf::from("docs").join("child"),
                docs: vec![
                    page("child/README.md", "Nested Root", None),
                    page("child/three.md", "Three", None),
                ],
                dirs: vec![Directory {
                    path: PathBuf::from("docs").join("child").join("nested"),
                    docs: vec![
                        page("child/nested/README.md", "Nested Root", None),
                        page("child/nested/four.md", "Four", None),
                    ],
                    dirs: vec![],
                }],
            }],
        };

        let rules = vec![
            NavRule::File(PathBuf::from("docs").join("one.md")),
            NavRule::Dir(
                PathBuf::from("docs").join("child"),
                Some(DirIncludeRule::Explicit(vec![NavRule::Dir(
                    PathBuf::from("docs").join("child").join("nested"),
                    Some(DirIncludeRule::Explicit(vec![NavRule::File(
                        PathBuf::from("docs")
                            .join("child")
                            .join("nested")
                            .join("four.md"),
                    )])),
                )])),
            ),
        ];

        let config = config(None);
        let navigation = Navigation::new(&config);
        let links: Vec<Link> = root.links();

        assert_eq!(
            navigation.customize(&rules, &links),
            vec![
                Link {
                    path: String::from("/one"),
                    title: String::from("One"),
                    children: vec![]
                },
                Link {
                    path: String::from("/child"),
                    title: String::from("Nested Root"),
                    children: vec![Link {
                        path: String::from("/child/nested"),
                        title: String::from("Nested Root"),
                        children: vec![Link {
                            path: String::from("/child/nested/four"),
                            title: String::from("Four"),
                            children: vec![]
                        },]
                    }]
                }
            ]
        );
    }

    #[test]
    fn manual_menu_file_from_nested_directory() {
        let root = Directory {
            path: PathBuf::from("docs"),
            docs: vec![page("README.md", "Getting Started", None)],
            dirs: vec![Directory {
                path: PathBuf::from("docs").join("child"),
                docs: vec![
                    page("child/README.md", "Nested Root", None),
                    page("child/three.md", "Three", None),
                ],
                dirs: vec![],
            }],
        };

        let rules = vec![NavRule::File(
            PathBuf::from("docs").join("child").join("three.md"),
        )];

        let config = config(None);
        let navigation = Navigation::new(&config);
        let links: Vec<Link> = root.links();

        assert_eq!(
            navigation.customize(&rules, &links),
            vec![Link {
                path: String::from("/child/three"),
                title: String::from("Three"),
                children: vec![]
            },]
        );
    }

    #[test]
    fn manual_menu_file_from_parent_directory() {
        let root = Directory {
            path: PathBuf::from("docs"),
            docs: vec![
                page("README.md", "Getting Started", None),
                page("one.md", "One", None),
            ],
            dirs: vec![Directory {
                path: PathBuf::from("docs").join("child"),
                docs: vec![page("child/README.md", "Nested Root", None)],
                dirs: vec![],
            }],
        };

        let rules = vec![NavRule::Dir(
            PathBuf::from("docs").join("child"),
            Some(DirIncludeRule::Explicit(vec![NavRule::File(
                PathBuf::from("docs").join("one.md"),
            )])),
        )];

        let config = config(None);
        let navigation = Navigation::new(&config);
        let links: Vec<Link> = root.links();

        assert_eq!(
            navigation.customize(&rules, &links),
            vec![Link {
                path: String::from("/child"),
                title: String::from("Nested Root"),
                children: vec![Link {
                    path: String::from("/one"),
                    title: String::from("One"),
                    children: vec![],
                }]
            },]
        );
    }

    #[test]
    fn build_with_base_path() {
        let config = config(Some(indoc! {"
        ---
        title: Not in the root
        base_path: /example
        "}));

        let root = Directory {
            path: PathBuf::from("docs"),
            docs: vec![
                page(
                    "README.md",
                    "Getting Started",
                    Some(config.base_path()),
                ),
                page("one.md", "One", Some(config.base_path())),
                page("two.md", "Two", Some(config.base_path())),
            ],
            dirs: vec![Directory {
                path: PathBuf::from("docs").join("child"),
                docs: vec![
                    page(
                        "child/README.md",
                        "Nested Root",
                        Some(config.base_path()),
                    ),
                    page("child/three.md", "Three", Some(config.base_path())),
                ],
                dirs: vec![],
            }],
        };

        let navigation = Navigation::new(&config);

        assert_eq!(
            navigation.build_for(&root),
            vec![
                Link {
                    path: String::from("/example/child"),
                    title: String::from("Nested Root"),
                    children: vec![Link {
                        path: String::from("/example/child/three"),
                        title: String::from("Three"),
                        children: vec![],
                    },],
                },
                Link {
                    path: String::from("/example/one"),
                    title: String::from("One"),
                    children: vec![],
                },
                Link {
                    path: String::from("/example/two"),
                    title: String::from("Two"),
                    children: vec![],
                },
            ]
        )
    }
}
