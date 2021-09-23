#[deny(clippy::all)]
#[cfg(test)]
#[macro_use]
extern crate indoc;

#[macro_use]
extern crate lazy_static;

mod build;
pub mod config;
mod error;
mod frontmatter;
mod init;
mod livereload_server;
mod navigation;
mod preview_server;
#[allow(dead_code, unused_variables)]
mod serve;
mod site;
mod site_generator;
mod watcher;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub use build::BuildCommand;
pub use config::Config;
pub use error::Error;
pub use init::InitCommand;
pub use serve::{ServeCommand, ServeOptions};
pub use site::BuildMode;

pub use doctave_markdown::{Heading, Markdown};
use handlebars::Handlebars;
use navigation::Link;
use serde::Serialize;

static APP_JS: &str = include_str!("assets/app.js");
static MERMAID_JS: &str = include_str!("assets/mermaid.min.js");
static ELASTIC_LUNR: &str = include_str!("assets/elasticlunr.min.js");
static LIVERELOAD_JS: &str = include_str!("assets/livereload.min.js");
static PRISM_JS: &str = include_str!("assets/prism.min.js");

static NORMALIZE_CSS: &str = include_str!("assets/normalize.css");
static ATOM_DARK_CSS: &str = include_str!("assets/prism-atom-dark.css");
static GH_COLORS_CSS: &str = include_str!("assets/prism-ghcolors.css");

lazy_static! {
    pub static ref HANDLEBARS: Handlebars<'static> = {
        let mut handlebars = Handlebars::new();

        handlebars
            .register_template_string("page", include_str!("../templates/page.html"))
            .unwrap();
        handlebars
            .register_template_string("navigation", include_str!("../templates/navigation.html"))
            .unwrap();
        handlebars
            .register_template_string("search", include_str!("../templates/search.html"))
            .unwrap();
        handlebars
            .register_template_string(
                "nested_navigation",
                include_str!("../templates/nested_navigation.html"),
            )
            .unwrap();
        handlebars
            .register_template_string("style.css", include_str!("../templates/style.css"))
            .unwrap();

        handlebars
    };
}

pub type Result<T> = std::result::Result<T, error::Error>;

#[derive(Debug, Clone)]
pub struct Directory {
    path: PathBuf,
    pub docs: Vec<Document>,
    pub dirs: Vec<Directory>,
}

impl Directory {
    fn path(&self) -> &Path {
        &self.path
    }

    fn index(&self) -> &Document {
        &self
            .docs
            .iter()
            .find(|d| d.original_file_name() == Some(OsStr::new("README.md")))
            .expect("No index file found for directory")
    }

    #[allow(unused)]
    fn traverse_documents(&self) -> Box<dyn Iterator<Item = &Document> + '_> {
        Box::new(
            self.docs
                .iter()
                .chain(self.dirs.iter().flat_map(|d| d.traverse_documents())),
        )
    }

    fn traverse_documents_mut(&mut self) -> Box<dyn Iterator<Item = &mut Document> + '_> {
        Box::new(
            self.docs.iter_mut().chain(
                self.dirs
                    .iter_mut()
                    .flat_map(|d| d.traverse_documents_mut()),
            ),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BidirectionalLinkEnd {
    pub linking_page_path: PathBuf,
    pub linking_page_title: String,
}

use std::sync::atomic::AtomicU32;

static DOCUMENT_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub id: u32,
    /// The relative path in the docs folder to the file
    path: PathBuf,
    rename: Option<String>,
    raw: String,
    markdown: Markdown,
    frontmatter: BTreeMap<String, String>,
    incoming_links: Vec<BidirectionalLinkEnd>,
}

impl Document {
    /// Loads a document from disk and parses it.
    ///
    /// Must be provided both the absolute path to the file, and the relative
    /// path inside the docs directory to the original file.
    fn load(absolute_path: &Path, relative_docs_path: &Path) -> Self {
        let raw = fs::read_to_string(absolute_path).unwrap();
        let frontmatter =
            frontmatter::parse(&raw).expect("TODO: Print an error when frontmatter is busted");

        Document::new(relative_docs_path, raw, frontmatter)
    }

    /// Creates a new document from its raw components
    fn new(path: &Path, raw: String, frontmatter: BTreeMap<String, String>) -> Self {
        let rename = if path.ends_with("README.md") {
            Some("index".to_string())
        } else {
            None
        };

        let markdown = doctave_markdown::parse(frontmatter::without(&raw), None);

        Document {
            id: DOCUMENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            path: path.to_path_buf(),
            raw,
            markdown,
            rename,
            frontmatter,
            incoming_links: Vec::new(),
        }
    }

    fn original_file_name(&self) -> Option<&OsStr> {
        self.path.file_name()
    }

    fn destination(&self, out: &Path) -> PathBuf {
        out.join(self.html_path())
    }

    fn html_path(&self) -> PathBuf {
        // TODO(Nik): Refactor this mess to be readable
        match self.rename {
            None => self.path.with_file_name(&format!(
                "{}.html",
                self.path.file_stem().unwrap().to_str().unwrap()
            )),
            Some(ref rename) => self.path.with_file_name(&format!("{}.html", rename)),
        }
    }

    fn uri_path(&self) -> PathBuf {
        Link::path_to_uri(&self.html_path())
    }

    fn markdown_section(&self) -> &str {
        frontmatter::without(&self.raw)
    }

    fn headings(&self) -> &[Heading] {
        &self.markdown.headings
    }

    fn links(&self) -> &[doctave_markdown::Link] {
        &self.markdown.links
    }

    fn incoming_links(&self) -> &[BidirectionalLinkEnd] {
        &self.incoming_links
    }

    fn add_incoming_link(&mut self, link: BidirectionalLinkEnd) {
        self.incoming_links.push(link)
    }

    fn html(&self) -> &str {
        &self.markdown.as_html
    }

    fn title(&self) -> &str {
        self.frontmatter
            .get("title")
            .map(|t| t.as_ref())
            .unwrap_or_else(|| self.path.file_stem().unwrap().to_str().unwrap())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn traverses_documents() {
        let root = Directory {
            path: PathBuf::from("foo"),
            docs: vec![Document::new(
                Path::new("foo/1"),
                String::new(),
                BTreeMap::new(),
            )],
            dirs: vec![
                Directory {
                    path: PathBuf::from("foo/bar"),
                    docs: vec![
                        Document::new(Path::new("foo/bar/2"), String::new(), BTreeMap::new()),
                        Document::new(Path::new("foo/bar/3"), String::new(), BTreeMap::new()),
                    ],
                    dirs: vec![

                Directory {
                    path: PathBuf::from("foo/bar"),
                    docs: vec![
                        Document::new(Path::new("foo/bar/baz/4"), String::new(), BTreeMap::new()),
                        Document::new(Path::new("foo/bar/baz/5"), String::new(), BTreeMap::new()),
                    ],
                    dirs: vec![],
                },
                    ],
                },
                Directory {
                    path: PathBuf::from("foo/bar"),
                    docs: vec![
                        Document::new(Path::new("foo/bar/6"), String::new(), BTreeMap::new()),
                        Document::new(Path::new("foo/bar/7"), String::new(), BTreeMap::new()),
                    ],
                    dirs: vec![],
                },
            ],
        };

        let expected: Vec<String> = vec![
            "1".to_owned(),
            "2".to_owned(),
            "3".to_owned(),
            "4".to_owned(),
            "5".to_owned(),
            "6".to_owned(),
            "7".to_owned(),
        ];

        assert_eq!(
            root.traverse_documents()
                .map(|d| d.title())
                .collect::<Vec<_>>(),
            expected
        );
    }
}
