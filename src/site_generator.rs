use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use elasticlunr::Index;
use rayon::prelude::*;
use serde::Serialize;
use walkdir::WalkDir;

use crate::config::Config;
use crate::navigation::{Link, Navigation};
use crate::site::{BuildMode, Site};
use crate::{Directory, Document};
use crate::{Error, Result};

static INCLUDE_DIR: &str = "_include";
static HEAD_FILE: &str = "_head.html";

pub struct SiteGenerator<'a, T: Site> {
    config: Config,
    site: Box<&'a T>,
    timestamp: String,
}

impl<'a, T: Site> SiteGenerator<'a, T> {
    pub fn new(site: &'a T) -> Self {
        let start = SystemTime::now();

        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        SiteGenerator {
            site: Box::new(site),
            config: site.config().clone(),
            timestamp: format!("{}", since_the_epoch.as_secs()),
        }
    }

    pub fn run(&self) -> Result<()> {
        let root = self.find_docs();
        let nav_builder = Navigation::new(&self.config);
        let navigation = nav_builder.build_for(&root);

        self.site.reset()?;

        let head_include = self.read_head_include()?;

        self.build_includes()?;
        self.build_assets()?;
        self.build_directory(&root, &navigation, head_include.as_deref())?;
        self.build_search_index(&root)?;

        Ok(())
    }

    fn read_head_include(&self) -> Result<Option<String>> {
        let custom_head = self.config.docs_dir().join(INCLUDE_DIR).join(HEAD_FILE);

        if custom_head.exists() {
            let content = fs::read_to_string(custom_head)
                .map_err(|e| Error::io(e, "Could not read custom head include file"))?;

            Ok(Some(content))
        } else {
            Ok(None)
        }
    }

    /// Copies over all custom includes from the _includes directory
    fn build_includes(&self) -> Result<()> {
        let custom_assets_dir = self.config.docs_dir().join(INCLUDE_DIR);

        for asset in WalkDir::new(&custom_assets_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter(|e| e.path().file_name() != Some(OsStr::new(HEAD_FILE)))
        {
            let stripped_path = asset
                .path()
                .strip_prefix(&custom_assets_dir)
                .expect("asset directory was not parent of found asset");

            let destination = self.config.out_dir().join(stripped_path);

            self.site.copy_file(asset.path(), &destination)?;
        }

        Ok(())
    }

    /// Builds fixed assets required by Doctave
    fn build_assets(&self) -> Result<()> {
        // Add JS
        self.site
            .add_file(
                &self.config.out_dir().join("assets").join("mermaid.js"),
                crate::MERMAID_JS.into(),
            )
            .map_err(|e| Error::io(e, "Could not write mermaid.js to assets directory"))?;
        self.site
            .add_file(
                &self.config.out_dir().join("assets").join("elasticlunr.js"),
                crate::ELASTIC_LUNR.into(),
            )
            .map_err(|e| Error::io(e, "Could not write elasticlunr.js to assets directory"))?;
        if let BuildMode::Dev = self.config.build_mode() {
            // Livereload only in release mode
            self.site
                .add_file(
                    &self.config.out_dir().join("assets").join("livereload.js"),
                    crate::LIVERELOAD_JS.into(),
                )
                .map_err(|e| Error::io(e, "Could not write livereload.js to assets directory"))?;
        }
        self.site
            .add_file(
                &self.config.out_dir().join("assets").join("prism.js"),
                crate::PRISM_JS.into(),
            )
            .map_err(|e| Error::io(e, "Could not write prism.js to assets directory"))?;
        self.site
            .add_file(
                &self.config.out_dir().join("assets").join("doctave-app.js"),
                crate::APP_JS.into(),
            )
            .map_err(|e| Error::io(e, "Could not write doctave-app.js to assets directory"))?;

        // Add styles
        self.site
            .add_file(
                &self
                    .config
                    .out_dir()
                    .join("assets")
                    .join("prism-atom-dark.css"),
                crate::ATOM_DARK_CSS.into(),
            )
            .map_err(|e| Error::io(e, "Could not write prism-atom-dark.css to assets directory"))?;
        self.site
            .add_file(
                &self
                    .config
                    .out_dir()
                    .join("assets")
                    .join("prism-ghcolors.css"),
                crate::GH_COLORS_CSS.into(),
            )
            .map_err(|e| Error::io(e, "Could not write prism-ghcolors.css to assets directory"))?;
        self.site
            .add_file(
                &self.config.out_dir().join("assets").join("normalize.css"),
                crate::NORMALIZE_CSS.into(),
            )
            .map_err(|e| Error::io(e, "Could not write normalize.css to assets directory"))?;

        let mut data = serde_json::Map::new();
        data.insert(
            "theme_main".to_string(),
            serde_json::Value::String(self.config.main_color().to_css_string()),
        );
        data.insert(
            "theme_main_dark".to_string(),
            serde_json::Value::String(self.config.main_color_dark().to_css_string()),
        );

        let mut out = Vec::new();

        crate::HANDLEBARS
            .render_to_write("style.css", &data, &mut out)
            .map_err(|e| Error::handlebars(e, "Could not write custom style sheet"))?;

        let destination = self
            .config
            .out_dir()
            .join("assets")
            .join("doctave-style.css");

        self.site.add_file(&destination, out.into())?;

        Ok(())
    }

    fn build_directory(
        &self,
        dir: &Directory,
        nav: &[Link],
        head_include: Option<&str>,
    ) -> Result<()> {
        let results: Result<Vec<()>> = dir
            .docs
            .par_iter()
            .map(|doc| {
                let page_title = if doc.uri_path() == "/" {
                    self.config.title().to_string()
                } else {
                    doc.title().to_string()
                };

                let data = TemplateData {
                    content: doc.html().to_string(),
                    headings: doc
                        .headings()
                        .iter()
                        .map(|heading| {
                            let mut map = BTreeMap::new();
                            map.insert("title", heading.title.clone());
                            map.insert("anchor", heading.anchor.clone());
                            map.insert("level", heading.level.to_string());

                            map
                        })
                        .collect::<Vec<_>>(),
                    navigation: &nav,
                    current_path: doc.uri_path(),
                    project_title: self.config.title().to_string(),
                    logo: self.config.logo().map(|l| l.to_string()),
                    build_mode: self.config.build_mode().to_string(),
                    base_path: self.config.base_path().to_owned(),
                    timestamp: &self.timestamp,
                    page_title,
                    head_include,
                };

                let mut out = Vec::new();

                crate::HANDLEBARS
                    .render_to_write("page", &data, &mut out)
                    .map_err(|e| Error::handlebars(e, "Could not render template"))?;

                self.site
                    .add_file(&doc.destination(self.config.out_dir()), out.into())?;

                Ok(())
            })
            .collect();
        let _ok = results?;

        dir.dirs
            .par_iter()
            .map(|d| self.build_directory(&d, &nav, head_include))
            .collect()
    }

    fn build_search_index(&self, root: &Directory) -> Result<()> {
        let mut index = Index::new(&["title", "uri", "body"]);

        self.build_search_index_for_dir(root, &mut index);

        {
            self.site
                .add_file(
                    &self.config.out_dir().join("search_index.json"),
                    index.to_json().as_bytes().into(),
                )
                .map_err(|e| Error::io(e, "Could not create search index"))
        }
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

    fn find_docs(&self) -> Directory {
        let mut root_dir = self.walk_dir(self.config.docs_dir()).unwrap_or(Directory {
            path: self.config.docs_dir().to_path_buf(),
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

                docs.push(Document::load(entry.path(), path, self.config.base_path()));
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
            self.config.base_path(),
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TemplateData<'a> {
    pub content: String,
    pub headings: Vec<BTreeMap<&'static str, String>>,
    pub navigation: &'a [Link],
    pub head_include: Option<&'a str>,
    pub current_path: String,
    pub page_title: String,
    pub base_path: String,
    pub logo: Option<String>,
    pub project_title: String,
    pub build_mode: String,
    pub timestamp: &'a str,
}
