#[cfg(test)]
#[macro_use]
extern crate indoc;

#[macro_use]
extern crate lazy_static;

mod frontmatter;
mod navigation;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use pulldown_cmark::{html, Options, Parser};
use serde::Serialize;
use walkdir::WalkDir;

use handlebars::Handlebars;
use navigation::Level;

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
    };
}

pub struct InitCommand {
    project_root: PathBuf,
    docs_root: PathBuf,
}

impl InitCommand {
    pub fn run(project_root: PathBuf) -> io::Result<()> {
        let docs_root = project_root.join("docs");

        let cmd = InitCommand { project_root, docs_root };

        cmd.create_readme()?;
        cmd.create_docs_dir()?;

        Ok(())
    }

    fn create_readme(&self) -> io::Result<()> {
        if !self.project_root.join("README.md").exists() {
            let mut file = File::create(self.project_root.join("README.md"))?;
            file.write(b"Hello, world\n============\n")?;
        }

        Ok(())
    }

    fn create_docs_dir(&self) -> io::Result<()> {
        if !self.project_root.join("docs").exists() {
            fs::create_dir(&self.docs_root)?;
        }

        Ok(())
    }
}

pub struct BuildCommand {
    project_root: PathBuf,
    docs_root: PathBuf,
    out: PathBuf,
}

impl BuildCommand {
    pub fn run(root: PathBuf) -> io::Result<()> {
        let cmd = BuildCommand {
            project_root: root.clone(),
            docs_root: root.join("docs"),
            out: root.join("site"),
        };

        cmd.reset_site_dir()?;
        cmd.build_site()?;

        Ok(())
    }

    fn reset_site_dir(&self) -> io::Result<()> {
        if self.project_root.join("site").exists() {
            fs::remove_dir_all(self.project_root.join("site"))?;
        }

        fs::create_dir(self.project_root.join("site"))?;

        Ok(())
    }

    fn build_site(&self) -> io::Result<()> {
        let root = self.find_docs();
        let navigation = Level::from(&root);

        println!("{:?}", navigation);

        self.build_directory(&root, &navigation)?;

        Ok(())
    }

    fn build_directory(&self, dir: &Directory, nav: &Level) -> io::Result<()> {
        fs::create_dir_all(dir.destination(&self.out))?;

        for doc in &dir.docs {
            let mut file = File::create(doc.destination(&self.out))?;

            let data = TemplateData {
                content: doc.html(),
                navigation: &nav,
            };

            HANDLEBARS
                .render_to_write("page", &data, &mut file)
                .unwrap();
        }

        for dir in &dir.dirs {
            self.build_directory(&dir, &nav)?;
        }

        Ok(())
    }

    fn find_docs(&self) -> Directory {
        let mut root_dir = self.walk_dir(self.docs_root.join("")).unwrap_or(Directory {
            docs: vec![],
            dirs: vec![],
        });

        println!("{:?}", root_dir);

        // Set doc directory's root README with the repo's root readme
        // if one didn't exist
        if let None = root_dir
            .docs
            .iter()
            .find(|doc| doc.original_file_name() == Some(OsStr::new("README.md")))
        {
            root_dir.docs.push(Document::load(&self.project_root, "README.md"));
        }

        root_dir
    }

    fn walk_dir<P: AsRef<Path>>(&self, dir: P) -> Option<Directory> {
        let mut docs = vec![];
        let mut dirs = vec![];

        for entry in WalkDir::new(dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                docs.push(Document::load(&self.docs_root, path));
            } else {
                if docs.is_empty() {
                    continue;
                } else {
                    if let Some(dir) = self.walk_dir(entry.into_path().as_path()) {
                        dirs.push(dir);
                    }
                }
            }
        }

        if docs.is_empty() {
            None
        } else {
            Some(Directory { docs, dirs })
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct TemplateData<'a> {
    content: String,
    navigation: &'a Level,
}

#[derive(Debug, Clone)]
struct Directory {
    docs: Vec<Document>,
    dirs: Vec<Directory>,
}

impl Directory {
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

#[derive(Debug, Clone)]
struct Document {
    path: PathBuf,
    root: PathBuf,
    rename: Option<String>,
    raw: String,
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

        Document {
            path: path.into(),
            root: root.into(),
            raw,
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

    fn html(&self) -> String {
        let raw_markdown = &self.raw[frontmatter::end_pos(&self.raw)..];

        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(&raw_markdown, options);

        // Write to String buffer.
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);

        html_output
    }

    fn title(&self) -> &str {
        self.frontmatter
            .get("title")
            .map(|t| t.as_ref())
            .unwrap_or(self.path.file_stem().unwrap().to_str().unwrap())
    }
}
