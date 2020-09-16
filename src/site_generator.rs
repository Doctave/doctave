use std::ffi::OsStr;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use crate::navigation::Level;
use crate::site::Site;
use crate::{Directory, Document};

use serde::Serialize;
use walkdir::WalkDir;

pub struct SiteGenerator<'a> {
    project_root: PathBuf,
    docs_dir: PathBuf,
    out_dir: PathBuf,
    site: &'a Site,
}

impl<'a> SiteGenerator<'a> {
    pub fn new<P, D, O>(site: &'a Site, project_root: P, docs_dir: D, out_dir: O) -> Self
    where
        P: Into<PathBuf>,
        D: Into<PathBuf>,
        O: Into<PathBuf>,
    {
        SiteGenerator {
            site,
            project_root: project_root.into(),
            docs_dir: docs_dir.into(),
            out_dir: out_dir.into(),
        }
    }

    pub fn run(&self) -> io::Result<()> {
        let root = self.find_docs(&self.project_root);
        let navigation = Level::from(&root);

        self.site.reset()?;

        self.build_directory(&root, &navigation)?;
        self.build_assets()?;

        Ok(())
    }

    fn build_assets(&self) -> io::Result<()> {
        fs::create_dir_all(self.out_dir.join("assets"))?;

        // Dump mermaid.js into the assets directory
        fs::write(
            self.out_dir.join("assets").join("mermaid.js"),
            crate::MERMAID_JS,
        )?;
        // Add styles
        fs::write(
            self.out_dir.join("assets").join("normalize.css"),
            crate::NORMALIZE_CSS,
        )?;
        fs::write(self.out_dir.join("assets").join("style.css"), crate::STYLES)?;

        Ok(())
    }

    fn build_directory(&self, dir: &Directory, nav: &Level) -> io::Result<()> {
        fs::create_dir_all(dir.destination(&self.out_dir))?;

        for doc in &dir.docs {
            let mut file = File::create(doc.destination(&self.out_dir))?;

            let data = TemplateData {
                content: doc.html(),
                navigation: &nav,
            };

            crate::HANDLEBARS
                .render_to_write("page", &data, &mut file)
                .unwrap();
        }

        for dir in &dir.dirs {
            self.build_directory(&dir, &nav)?;
        }

        Ok(())
    }

    fn find_docs(&self, project_root: &Path) -> Directory {
        let mut root_dir = self
            .walk_dir(project_root.join("docs"))
            .unwrap_or(Directory {
                docs: vec![],
                dirs: vec![],
            });

        // Set doc directory's root README with the repo's root readme
        // if one didn't exist
        if let None = root_dir
            .docs
            .iter()
            .find(|doc| doc.original_file_name() == Some(OsStr::new("README.md")))
        {
            root_dir
                .docs
                .push(Document::load(project_root, "README.md"));
        }

        root_dir
    }

    fn walk_dir<P: AsRef<Path>>(&self, dir: P) -> Option<Directory> {
        let mut docs = vec![];
        let mut dirs = vec![];

        let current_dir: &Path = dir.as_ref();

        for entry in WalkDir::new(&current_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                docs.push(Document::load(&self.docs_dir, path));
            } else {
                let path = entry.into_path();

                if path.as_path() == current_dir {
                    continue;
                }

                if let Some(dir) = self.walk_dir(path) {
                    dirs.push(dir);
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
pub struct TemplateData<'a> {
    pub content: String,
    pub navigation: &'a Level,
}
