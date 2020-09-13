use std::ffi::OsStr;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use crate::navigation::Level;
use crate::{Directory, Document};

use serde::Serialize;
use walkdir::WalkDir;

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

            crate::HANDLEBARS
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

        // Set doc directory's root README with the repo's root readme
        // if one didn't exist
        if let None = root_dir
            .docs
            .iter()
            .find(|doc| doc.original_file_name() == Some(OsStr::new("README.md")))
        {
            root_dir
                .docs
                .push(Document::load(&self.project_root, "README.md"));
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

                docs.push(Document::load(&self.docs_root, path));
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
struct TemplateData<'a> {
    content: String,
    navigation: &'a Level,
}
