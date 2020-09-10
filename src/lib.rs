#[cfg(test)]
#[macro_use]
extern crate indoc;

#[macro_use]
extern crate lazy_static;

mod frontmatter;
mod navigation;

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use pulldown_cmark::{html, Options, Parser};
use walkdir::WalkDir;
use serde::Serialize;

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
    root: PathBuf,
}

impl InitCommand {
    pub fn run(root: PathBuf) -> io::Result<()> {
        let cmd = InitCommand { root };

        cmd.create_readme()?;
        cmd.create_docs_dir()?;

        Ok(())
    }

    fn create_readme(&self) -> io::Result<()> {
        if !self.root.join("README.md").exists() {
            let mut file = File::create(self.root.join("README.md"))?;
            file.write(b"Hello, world\n============\n")?;
        }

        Ok(())
    }

    fn create_docs_dir(&self) -> io::Result<()> {
        if !self.root.join("docs").exists() {
            fs::create_dir(self.root.join("docs"))?;
        }

        Ok(())
    }
}

pub struct BuildCommand {
    root: PathBuf,
    out: PathBuf,
}

impl BuildCommand {
    pub fn run(root: PathBuf) -> io::Result<()> {
        let cmd = BuildCommand {
            root: root.clone(),
            out: root.join("site"),
        };

        cmd.reset_site_dir()?;
        cmd.build_site()?;

        Ok(())
    }

    fn reset_site_dir(&self) -> io::Result<()> {
        if self.root.join("site").exists() {
            fs::remove_dir_all(self.root.join("site"))?;
        }

        fs::create_dir(self.root.join("site"))?;

        Ok(())
    }

    fn build_site(&self) -> io::Result<()> {
        let mut index = Document::new(&self.root, "README.md");
        index.rename("index");

        let mut documents = vec![index];

        for entry in WalkDir::new("docs")
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let f_name = entry.file_name().to_string_lossy();

            if f_name.ends_with(".md") {
                documents.push(Document::new(&self.root, entry.path().to_path_buf()));
            }
        }

        let destinations = documents
            .iter()
            .map(|doc| doc.link())
            .collect::<Vec<_>>();

        let navigation = navigation::build(&destinations);

        for page in documents {
            fs::create_dir_all(page.destination(&self.out).parent().unwrap())?;
            let mut file = File::create(page.destination(&self.out))?;

            let data = TemplateData {
                content: page.html(),
                navigation: &navigation,
            };

            println!("{:?}", data.navigation);

            HANDLEBARS
                .render_to_write("page", &data, &mut file)
                .unwrap();
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
struct TemplateData<'a> {
    content: String,
    navigation: &'a Level,
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
    fn new<R: Into<PathBuf>, S: Into<PathBuf>>(root: R, markdown_source: S) -> Self {
        let root = root.into();
        let mut path = markdown_source.into();

        if path.starts_with(&root) {
            path = path.strip_prefix(&root).unwrap().to_path_buf();
        }

        let raw = fs::read_to_string(root.join(&path)).unwrap();
        let frontmatter =
            frontmatter::parse(&raw).expect("TODO: Print an error when frontmatter is busted");

        Document {
            path,
            root,
            raw,
            rename: None,
            frontmatter,
        }
    }

    fn link(&self) -> (PathBuf, String) {
        (self.relative_path(), self.title().to_string())
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

    fn rename(&mut self, new_name: &str) {
        self.rename = Some(new_name.to_string());
    }

    fn title(&self) -> &str {
        self.frontmatter
            .get("title")
            .map(|t| t.as_ref())
            .unwrap_or(self.path.file_stem().unwrap().to_str().unwrap())
    }
}
