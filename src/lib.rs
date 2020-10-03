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
mod markdown;
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
pub use error::Error;
pub use init::InitCommand;
pub use markdown::{Heading, Markdown};
pub use serve::ServeCommand;

use handlebars::Handlebars;

static APP_JS: &'static str = include_str!("assets/app.js");
static MERMAID_JS: &'static str = include_str!("assets/mermaid.min.js");
static ELASTIC_LUNR: &'static str = include_str!("assets/elasticlunr.min.js");
static LIVERELOAD_JS: &'static str = include_str!("assets/livereload.min.js");
static PRISM_JS: &'static str = include_str!("assets/prism.min.js");

static NORMALIZE_CSS: &'static str = include_str!("assets/normalize.css");
static ATOM_DARK_CSS: &'static str = include_str!("assets/prism-atom-dark.css");
static GH_COLORS_CSS: &'static str = include_str!("assets/prism-ghcolors.css");

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
    docs: Vec<Document>,
    dirs: Vec<Directory>,
}

impl Directory {
    fn path(&self) -> &Path {
        &self.path
    }

    fn destination(&self, out: &Path) -> PathBuf {
        self.docs
            .get(0)
            .unwrap()
            .destination(out)
            .parent()
            .unwrap()
            .to_path_buf()
    }
}

use std::sync::atomic::AtomicU32;

static DOCUMENT_ID: AtomicU32 = AtomicU32::new(1);

#[derive(Debug, Clone, PartialEq)]
struct Document {
    pub id: u32,
    /// The relative path in the docs folder to the file
    path: PathBuf,
    rename: Option<String>,
    raw: String,
    markdown: Markdown,
    frontmatter: BTreeMap<String, String>,
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

        let markdown = markdown::parse(frontmatter::without(&raw));

        Document {
            id: DOCUMENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            path: path.to_path_buf(),
            raw,
            markdown,
            rename,
            frontmatter,
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

    fn markdown_section(&self) -> &str {
        frontmatter::without(&self.raw)
    }

    fn headings(&self) -> &[Heading] {
        &self.markdown.headings
    }

    fn html(&self) -> &str {
        &self.markdown.as_html
    }

    fn title(&self) -> &str {
        self.frontmatter
            .get("title")
            .map(|t| t.as_ref())
            .unwrap_or(self.path.file_stem().unwrap().to_str().unwrap())
    }
}
