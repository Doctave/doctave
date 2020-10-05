use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use elasticlunr::Index;
use rayon::prelude::*;
use serde::Serialize;
use walkdir::WalkDir;

use crate::config::Config;
use crate::markdown::Heading;
use crate::navigation::{Link, Navigation};
use crate::site::Site;
use crate::{Directory, Document};
use crate::{Error, Result};

pub struct SiteGenerator<'a> {
    config: &'a Config,
    site: &'a Site,
}

impl<'a> SiteGenerator<'a> {
    pub fn new(config: &'a Config, site: &'a Site) -> Self {
        SiteGenerator { config, site }
    }

    pub fn run(&self) -> Result<()> {
        let root = self.find_docs(self.config.project_root());
        let nav_builder = Navigation::new(&self.config);
        let navigation = nav_builder.build_for(&root);

        self.site.reset()?;

        self.build_directory(&root, &navigation)?;
        self.build_search_index(&root)?;
        self.build_assets()?;

        Ok(())
    }

    fn build_assets(&self) -> Result<()> {
        fs::create_dir_all(self.config.out_dir().join("assets"))
            .map_err(|e| Error::io(e, "Could not create assets directory"))?;

        // Add JS
        fs::write(
            self.config.out_dir().join("assets").join("mermaid.js"),
            crate::MERMAID_JS,
        )
        .map_err(|e| Error::io(e, "Could not write mermaid.js to assets directory"))?;
        fs::write(
            self.config.out_dir().join("assets").join("elasticlunr.js"),
            crate::ELASTIC_LUNR,
        )
        .map_err(|e| Error::io(e, "Could not write elasticlunr.js to assets directory"))?;
        fs::write(
            self.config.out_dir().join("assets").join("livereload.js"),
            crate::LIVERELOAD_JS,
        )
        .map_err(|e| Error::io(e, "Could not write livereload.js to assets directory"))?;
        fs::write(
            self.config.out_dir().join("assets").join("prism.js"),
            crate::PRISM_JS,
        )
        .map_err(|e| Error::io(e, "Could not write prism.js to assets directory"))?;
        fs::write(
            self.config.out_dir().join("assets").join("doctave-app.js"),
            crate::APP_JS,
        )
        .map_err(|e| Error::io(e, "Could not write doctave-app.js to assets directory"))?;

        // Add styles
        fs::write(
            self.config
                .out_dir()
                .join("assets")
                .join("prism-atom-dark.css"),
            crate::ATOM_DARK_CSS,
        )
        .map_err(|e| Error::io(e, "Could not write prism-atom-dark.css to assets directory"))?;
        fs::write(
            self.config
                .out_dir()
                .join("assets")
                .join("prism-ghcolors.css"),
            crate::GH_COLORS_CSS,
        )
        .map_err(|e| Error::io(e, "Could not write prism-ghcolors.css to assets directory"))?;
        fs::write(
            self.config.out_dir().join("assets").join("normalize.css"),
            crate::NORMALIZE_CSS,
        )
        .map_err(|e| Error::io(e, "Could not write normalize.css to assets directory"))?;

        let mut style = File::create(
            self.config
                .out_dir()
                .join("assets")
                .join("doctave-style.css"),
        )
        .map_err(|e| Error::io(e, "Could not create doctave-style.css in assets directory"))?;

        let mut data = serde_json::Map::new();
        data.insert(
            "theme_main".to_string(),
            serde_json::Value::String(self.config.main_color().to_css_string()),
        );
        data.insert(
            "theme_main_dark".to_string(),
            serde_json::Value::String(self.config.main_color_dark().to_css_string()),
        );

        crate::HANDLEBARS
            .render_to_write("style.css", &data, &mut style)
            .unwrap();

        // Copy over all custom assets from the _assets directory
        let custom_assets_dir = self.config.docs_dir().join("_assets");

        for asset in WalkDir::new(&custom_assets_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let stripped_path = asset
                .path()
                .strip_prefix(&custom_assets_dir)
                .expect("asset directory was not parent of found asset");

            let destination = self.config.out_dir().join("assets").join(stripped_path);

            File::create(&destination)
                .map_err(|e| Error::io(e, "Could not create custom asset in assets directory"))?;
            fs::create_dir_all(
                asset
                    .path()
                    .parent()
                    .expect("asset did not have parent directory"),
            )
            .map_err(|e| Error::io(e, "Could not create custom asset parent directory"))?;
            fs::copy(asset.path(), destination)
                .map_err(|e| Error::io(e, "Could not copy custom asset"))?;
        }

        Ok(())
    }

    fn build_directory(&self, dir: &Directory, nav: &[Link]) -> Result<()> {
        fs::create_dir_all(dir.destination(self.config.out_dir()))
            .map_err(|e| Error::io(e, "Could not create site directory"))?;

        let results: Result<Vec<()>> = dir
            .docs
            .par_iter()
            .map(|doc| {
                let mut file =
                    File::create(doc.destination(self.config.out_dir())).map_err(|e| {
                        Error::io(
                            e,
                            format!(
                                "Could not create page {}",
                                doc.destination(self.config.out_dir()).display()
                            ),
                        )
                    })?;

                let page_title = if doc.uri_path() == "/" {
                    self.config.title().to_string()
                } else {
                    doc.title().to_string()
                };

                let data = TemplateData {
                    content: doc.html().to_string(),
                    headings: doc.headings().to_vec(),
                    navigation: &nav,
                    current_path: doc.uri_path(),
                    project_title: self.config.title().to_string(),
                    logo: self.config.logo(),
                    page_title,
                };

                crate::HANDLEBARS
                    .render_to_write("page", &data, &mut file)
                    .map_err(|e| Error::handlebars(e, "Could not render template"))?;

                Ok(())
            })
            .collect();
        let _ok = results?;

        dir.dirs
            .par_iter()
            .map(|d| self.build_directory(&d, &nav))
            .collect()
    }

    fn build_search_index(&self, root: &Directory) -> Result<()> {
        let mut index = Index::new(&["title", "uri", "body"]);

        self.build_search_index_for_dir(root, &mut index);

        fs::write(
            self.config.out_dir().join("search_index.json"),
            index.to_json().as_bytes(),
        )
        .map_err(|e| Error::io(e, "Could not create search index"))
    }

    fn build_search_index_for_dir(&self, root: &Directory, index: &mut Index) {
        for doc in &root.docs {
            index.add_doc(
                &doc.id.to_string(),
                &[
                    &doc.title(),
                    &doc.uri_path().as_str(),
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
                path: project_root.join("docs"),
                docs: vec![],
                dirs: vec![],
            });

        self.generate_missing_indices(&mut root_dir);

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
            if entry.file_type().is_file() && entry.path().extension() == Some(OsStr::new("md")) {
                let path = entry.path().strip_prefix(self.config.docs_dir()).unwrap();

                docs.push(Document::load(entry.path(), path));
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
            Some(Directory {
                path: current_dir.to_path_buf(),
                docs,
                dirs,
            })
        }
    }

    fn generate_missing_indices(&self, dir: &mut Directory) {
        if dir
            .docs
            .iter()
            .find(|d| d.original_file_name() == Some(OsStr::new("README.md")))
            .is_none()
        {
            let new_index = self.generate_missing_index(dir);
            dir.docs.push(new_index);
        }

        for mut child in &mut dir.dirs {
            self.generate_missing_indices(&mut child);
        }
    }

    fn generate_missing_index(&self, dir: &mut Directory) -> Document {
        let content = dir
            .docs
            .iter()
            .map(|d| format!("* [{}]({})", d.title(), d.uri_path()))
            .collect::<Vec<_>>()
            .join("\n");

        let mut frontmatter = BTreeMap::new();
        frontmatter.insert(
            "title".to_string(),
            format!("{}", dir.path().file_name().unwrap().to_string_lossy()),
        );

        let tmp = dir.path().join("README.md");
        let path = tmp.strip_prefix(self.config.docs_dir()).unwrap();

        Document::new(
            path,
            format!(
                "# Index of {}\n \
                \n \
                This page was generated automatically by Doctave, because the directory \
                `{}` did not contain an index `README.md` file. You can customize this page by \
                creating one yourself.\
                \n\
                ## Pages\n\
                \n\
                {}",
                dir.path().file_name().unwrap().to_string_lossy(),
                dir.path()
                    .strip_prefix(self.config.project_root())
                    .unwrap_or_else(|_| dir.path())
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
    pub navigation: &'a [Link],
    pub current_path: String,
    pub page_title: String,
    pub logo: Option<PathBuf>,
    pub project_title: String,
}
