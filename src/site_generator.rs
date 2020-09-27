use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io;
use std::path::Path;

use elasticlunr::Index;
use rayon::prelude::*;
use serde::Serialize;
use walkdir::WalkDir;

use crate::config::Config;
use crate::markdown::Heading;
use crate::navigation::{Level, Link};
use crate::site::Site;
use crate::{Directory, Document};

pub struct SiteGenerator<'a> {
    config: &'a Config,
    site: &'a Site,
}

impl<'a> SiteGenerator<'a> {
    pub fn new(config: &'a Config, site: &'a Site) -> Self {
        SiteGenerator { config, site }
    }

    pub fn run(&self) -> io::Result<()> {
        let root = self.find_docs(self.config.project_root());
        let navigation = Level::from(&root);

        self.site.reset()?;

        self.build_directory(&root, &navigation)?;
        self.build_search_index(&root)?;
        self.build_assets()?;

        Ok(())
    }

    fn build_assets(&self) -> io::Result<()> {
        fs::create_dir_all(self.config.out_dir().join("assets"))?;

        // Add JS
        fs::write(
            self.config.out_dir().join("assets").join("mermaid.js"),
            crate::MERMAID_JS,
        )?;
        fs::write(
            self.config.out_dir().join("assets").join("elasticlunr.js"),
            crate::ELASTIC_LUNR,
        )?;
        fs::write(
            self.config.out_dir().join("assets").join("livereload.js"),
            crate::LIVERELOAD_JS,
        )?;
        fs::write(
            self.config.out_dir().join("assets").join("app.js"),
            crate::APP_JS,
        )?;

        // Add styles
        fs::write(
            self.config.out_dir().join("assets").join("normalize.css"),
            crate::NORMALIZE_CSS,
        )?;
        fs::write(
            self.config.out_dir().join("assets").join("style.css"),
            crate::STYLES,
        )?;

        Ok(())
    }

    fn build_directory(&self, dir: &Directory, nav: &Level) -> io::Result<()> {
        fs::create_dir_all(dir.destination(&self.config.out_dir()))?;

        let results: Result<Vec<()>, io::Error> = dir
            .docs
            .par_iter()
            .map(|doc| {
                let mut file = File::create(doc.destination(&self.config.out_dir()))?;

                let page_title = if Link::from(doc).path == "/" {
                    self.config.title().to_string()
                } else {
                    doc.title().to_string()
                };

                let data = TemplateData {
                    content: doc.html().to_string(),
                    headings: doc.headings().to_vec(),
                    navigation: &nav,
                    current_page: Link::from(doc),
                    project_title: self.config.title().to_string(),
                    page_title,
                };

                crate::HANDLEBARS
                    .render_to_write("page", &data, &mut file)
                    .unwrap();

                Ok(())
            })
            .collect();
        let _ok = results?;

        dir.dirs
            .par_iter()
            .map(|d| self.build_directory(&d, &nav))
            .collect()
    }

    fn build_search_index(&self, root: &Directory) -> io::Result<()> {
        let mut index = Index::new(&["title", "uri", "body"]);

        self.build_search_index_for_dir(root, &mut index);

        fs::write(
            self.config.out_dir().join("search_index.json"),
            index.to_json().as_bytes(),
        )
    }

    fn build_search_index_for_dir(&self, root: &Directory, index: &mut Index) {
        for doc in &root.docs {
            index.add_doc(
                &doc.id.to_string(),
                &[
                    &doc.title(),
                    &Link::from(doc).path.as_str(),
                    doc.markdown_section(),
                ],
            );
        }
        for dir in &root.dirs {
            self.build_search_index_for_dir(&dir, index);
        }
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

        self.generate_missing_indices(&mut root_dir.dirs);

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

                docs.push(Document::load(&self.config.docs_dir(), path));
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

    fn generate_missing_indices(&self, dirs: &mut [Directory]) {
        for mut dir in dirs {
            if dir
                .docs
                .iter()
                .find(|d| d.original_file_name() == Some(OsStr::new("README.md")))
                .is_none()
            {
                let new_index = self.generate_missing_index(&mut dir);
                dir.docs.push(new_index);
            }
        }
    }

    fn generate_missing_index(&self, dir: &mut Directory) -> Document {
        let content = dir
            .docs
            .iter()
            .map(|d| format!("* [{}]({})", Link::from(d).title, Link::from(d).path,))
            .collect::<Vec<_>>()
            .join("\n");

        let mut frontmatter = BTreeMap::new();
        frontmatter.insert(
            "title".to_string(),
            format!("{}", dir.path().file_name().unwrap().to_string_lossy()),
        );

        Document::new(
            dir.path().join("README.md"),
            &self.config.docs_dir(),
            format!(
                "# Index of {}\n \
                \n \
                This page was generated automatically by Doctave, because the directory \
                `{}` did not contain an index `README.md` file. You can customize this page by \
                creating one yourself.\
                \n\
                # Pages\n\
                \n\
                {}",
                dir.path().file_name().unwrap().to_string_lossy(),
                dir.path()
                    .strip_prefix(self.config.project_root())
                    .unwrap_or(dir.path())
                    .display(),
                content
            ),
            frontmatter,
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateData<'a> {
    pub content: String,
    pub headings: Vec<Heading>,
    pub navigation: &'a Level,
    pub current_page: Link,
    pub page_title: String,
    pub project_title: String,
}
