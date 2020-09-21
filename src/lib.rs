#[cfg(test)]
#[macro_use]
extern crate indoc;

#[macro_use]
extern crate lazy_static;

mod build;
mod frontmatter;
mod init;
mod livereload_server;
mod preview_server;
mod markdown;
mod navigation;
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
pub use init::InitCommand;
pub use markdown::{Heading, Markdown};
pub use serve::ServeCommand;

use handlebars::Handlebars;

static APP_JS: &'static str = include_str!("assets/app.js");
static MERMAID_JS: &'static str = include_str!("assets/mermaid.min.js");
static ELASTIC_LUNR: &'static str = include_str!("assets/elasticlunr.min.js");
static LIVERELOAD_JS: &'static str = include_str!("assets/livereload.min.js");

static STYLES: &'static str = include_str!("assets/style.css");
static NORMALIZE_CSS: &'static str = include_str!("assets/normalize.css");

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
    };
}

#[derive(Debug, Clone)]
struct Directory {
    docs: Vec<Document>,
    dirs: Vec<Directory>,
}

impl Directory {
    fn path(&self) -> &Path {
        &self.docs[0].path.parent().unwrap()
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
    path: PathBuf,
    root: PathBuf,
    rename: Option<String>,
    raw: String,
    markdown: Markdown,
    frontmatter: BTreeMap<String, String>,
}

impl Document {
    /// Loads a markdown document from disk, given a root directory and the relative
    /// path inside the root directory.
    fn load<R: Into<PathBuf>, S: Into<PathBuf>>(root: R, markdown_source: S) -> Self {
        let root = root.into();
        let path = markdown_source.into();

        let raw = fs::read_to_string(root.join(&path)).unwrap();
        let frontmatter =
            frontmatter::parse(&raw).expect("TODO: Print an error when frontmatter is busted");

        Document::new(path, root, raw, frontmatter)
    }

    /// Creates a new document from its raw components
    fn new<P: Into<PathBuf>, R: Into<PathBuf>>(
        path: P,
        root: R,
        raw: String,
        frontmatter: BTreeMap<String, String>,
    ) -> Self {
        let root = root.into();
        let mut path = path.into();

        if path.starts_with(&root) {
            path = path.strip_prefix(&root).unwrap().to_path_buf();
        }

        let rename = if path.ends_with("README.md") {
            Some("index".to_string())
        } else {
            None
        };

        let markdown = markdown::parse(frontmatter::without(&raw));

        Document {
            id: DOCUMENT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
            path: path.into(),
            root: root.into(),
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
        out.join(self.relative_path())
    }

    fn relative_path(&self) -> PathBuf {
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
