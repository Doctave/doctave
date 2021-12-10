use crate::config::Config;
use crate::preview_server::resolve_file;
use crate::site::{Site, SiteBackend};
use crate::Directory;
use crate::{Error, Result};

use std::path::{Path, PathBuf};

pub fn run<B: SiteBackend>(site: &Site<B>) -> Result<()> {
    let mut broken_links = Vec::new();
    find_broken_links(&site.root, site, &mut broken_links, &site.config);

    if broken_links.len() == 0 {
        Ok(())
    } else {
        Err(Error::broken_links(broken_links))
    }
}

fn find_broken_links<B: SiteBackend>(
    dir: &Directory,
    site: &Site<B>,
    broken_links: &mut Vec<(PathBuf, doctave_markdown::Link)>,
    config: &Config,
) {
    for doc in &dir.docs {
        for link in doc.outgoing_links() {
            match &link.url {
                doctave_markdown::UrlType::Remote(_) => {}
                doctave_markdown::UrlType::Local(path) => {
                    if !matches_a_target(&path, site, config.base_path()) {
                        broken_links.push((doc.original_path().to_owned(), link.clone()))
                    }
                }
            }
        }
    }

    for child_dir in &dir.dirs {
        find_broken_links(child_dir, site, broken_links, config);
    }
}

fn matches_a_target<B: SiteBackend>(path: &Path, site: &Site<B>, base_path: &str) -> bool {
    resolve_file(path, &site, base_path).is_some()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::Config;
    use crate::Document;
    use std::collections::BTreeMap;

    fn page(path: &str, name: &str, content: &str) -> Document {
        let mut frontmatter = BTreeMap::new();
        frontmatter.insert("title".to_string(), name.to_string());

        Document::new(Path::new(path), content.to_string(), frontmatter, "/")
    }

    fn config(yaml: Option<&str>) -> Config {
        let conf = yaml.unwrap_or("---\ntitle: My project\n");

        Config::from_yaml_str(&Path::new("project"), conf).unwrap()
    }

    #[test]
    fn detects_broken_links() {
        let config = config(None);

        let root = Directory {
            path: config.docs_dir().to_path_buf(),
            docs: vec![page(
                "README.md",
                "Getting Started",
                "[highway to hell](/dont-exist)",
            )],
            dirs: vec![],
        };

        let site = Site::with_root(root, config);

        assert!(run(&site).is_err());
    }

    #[test]
    fn is_fine_if_no_broken_links_exist() {
        let config = config(None);

        let root = Directory {
            path: config.docs_dir().to_path_buf(),
            docs: vec![
                page("README.md", "Getting Started", "[highway to hell](/other)"),
                page("other.md", "Getting Started", "No links!"),
            ],
            dirs: vec![],
        };

        let site = Site::with_root(root, config);
        site.build().unwrap();
        let result = run(&site);

        println!("{:?}", result);

        assert!(result.is_ok());
    }

    #[test]
    fn does_not_mind_if_the_url_has_an_html_extension() {
        let config = config(None);

        let root = Directory {
            path: config.docs_dir().to_path_buf(),
            docs: vec![
                page(
                    "README.md",
                    "Getting Started",
                    "[highway to hell](/other.html)",
                ),
                page("other.md", "Getting Started", "No links!"),
            ],
            dirs: vec![],
        };

        let site = Site::with_root(root, config);
        site.build().unwrap();
        let result = run(&site);

        println!("{:?}", result);

        assert!(result.is_ok());
    }

    #[test]
    fn handles_files_in_subdirectories() {
        let config = config(None);

        let root = Directory {
            path: config.docs_dir().to_path_buf(),
            docs: vec![
                page(
                    "README.md",
                    "Getting Started",
                    "[I'm on a](/nested/)\n[highway to hell](/nested/other.html)",
                ),
                page("other.md", "Getting Started", "No links!"),
            ],
            dirs: vec![Directory {
                path: config.docs_dir().to_path_buf().join("nested"),
                docs: vec![
                    page("nested/README.md", "Nested", "Content"),
                    page("nested/other.md", "Nested Child", "No links!"),
                ],
                dirs: vec![],
            }],
        };

        let site = Site::with_root(root, config);
        site.build().unwrap();
        let result = run(&site);

        println!("{:?}", result);

        assert!(result.is_ok());
    }
}
