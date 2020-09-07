use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use pulldown_cmark::{html, Options, Parser};

use walkdir::WalkDir;

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
        let mut index = Page::new(&self.root, "README.md");
        index.rename("index");

        let mut pages = vec![index];

        for entry in WalkDir::new("docs")
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let f_name = entry.file_name().to_string_lossy();

            if f_name.ends_with(".md") {
                pages.push(Page::new(&self.root, entry.path().to_path_buf()));
            }
        }

        for page in pages {
            let mut file = File::create(page.destination(&self.out))?;
            file.write(page.content().as_bytes())?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Page {
    path: PathBuf,
    root: PathBuf,
    rename: Option<String>,
}

impl Page {
    fn new<R: Into<PathBuf>, S: Into<PathBuf>>(root: R, markdown_source: S) -> Self {
        let root = root.into();
        let mut path = markdown_source.into();

        if path.starts_with(&root) {
            path = path.strip_prefix(&root).unwrap().to_path_buf();
        }

        Page {
            path,
            root,
            rename: None,
        }
    }

    fn destination(&self, out: &Path) -> PathBuf {
        match self.rename {
            None => out.join(&self.path.with_file_name(&format!(
                "{}.html",
                self.path.file_stem().unwrap().to_str().unwrap()
            ))),
            Some(ref rename) => out.join(&self.path.with_file_name(&format!("{}.html", rename))),
        }
    }

    fn content(&self) -> String {
        let raw_markdown = fs::read_to_string(&self.root.join(&self.path)).unwrap();

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
}
