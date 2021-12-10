use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use walkdir::WalkDir;

use crate::broken_links_checker;
use crate::config::Config;
use crate::site_generator::SiteGenerator;
use crate::{Directory, Document};
use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq)]
/// Describes the mode we should build the site in, meaning
/// which assets we want to include/exclude for development.
pub enum BuildMode {
    Dev,
    Release,
}

impl std::fmt::Display for BuildMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildMode::Dev => write!(f, "dev"),
            BuildMode::Release => write!(f, "release"),
        }
    }
}

#[derive(Debug, Clone)]
/// The main handle to a site. Generic over a backend implementation.
/// Currently has InMemory and DiskBacked backends, used for serve and build respectively.
///
/// When `build` is called on this struct, the backend is populated by the
/// `SiteGenerator`.
pub struct Site<B: SiteBackend> {
    pub root: Directory,
    pub backend: B,
    pub config: Config,
}

impl Site<InMemorySite> {
    pub fn in_memory(config: Config) -> Site<InMemorySite> {
        let root = Self::find_docs(&config);

        Site {
            backend: InMemorySite::new(config.clone()),
            root,
            config,
        }
    }

    pub fn with_root(root: Directory, config: Config) -> Site<InMemorySite> {
        Site {
            backend: InMemorySite::new(config.clone()),
            root,
            config,
        }
    }
}

impl Site<DiskBackedSite> {
    pub fn disk_backed(config: Config) -> Site<DiskBackedSite> {
        let root = Self::find_docs(&config);

        Site {
            backend: DiskBackedSite::new(config.clone()),
            root,
            config,
        }
    }
}

impl<B: SiteBackend> Site<B> {
    pub fn build(&self) -> Result<()> {
        self.backend.build(&self.root)
    }

    pub fn check_dead_links(&self) -> Result<()> {
        broken_links_checker::run(&self)
    }

    fn find_docs(config: &Config) -> Directory {
        let mut root_dir = Self::walk_dir(config.docs_dir(), config).unwrap_or(Directory {
            path: config.docs_dir().to_path_buf(),
            docs: vec![],
            dirs: vec![],
        });

        Self::generate_missing_indices(&mut root_dir, config);

        root_dir
    }

    fn walk_dir<P: AsRef<Path>>(dir: P, config: &Config) -> Option<Directory> {
        let mut docs = vec![];
        let mut dirs = vec![];

        let current_dir: &Path = dir.as_ref();

        for entry in WalkDir::new(&current_dir)
            .max_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() && entry.path().extension() == Some(OsStr::new("md")) {
                let path = entry.path().strip_prefix(config.docs_dir()).unwrap();

                docs.push(Document::load(entry.path(), path, config.base_path()));
            } else {
                let path = entry.into_path();

                if path.as_path() == current_dir {
                    continue;
                }

                if let Some(dir) = Self::walk_dir(path, config) {
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

    fn generate_missing_indices(dir: &mut Directory, config: &Config) {
        if dir
            .docs
            .iter()
            .find(|d| d.original_file_name() == Some(OsStr::new("README.md")))
            .is_none()
        {
            let new_index = Self::generate_missing_index(dir, config);
            dir.docs.push(new_index);
        }

        for mut child in &mut dir.dirs {
            Self::generate_missing_indices(&mut child, config);
        }
    }

    fn generate_missing_index(dir: &mut Directory, config: &Config) -> Document {
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
        let path = tmp.strip_prefix(config.docs_dir()).unwrap();

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
                    .strip_prefix(config.project_root())
                    .unwrap_or_else(|_| dir.path())
                    .display(),
                content
            ),
            frontmatter,
            config.base_path(),
        )
    }
}

pub trait SiteBackend: Send + Sync {
    fn config(&self) -> &Config;
    fn add_file(&self, path: &Path, content: Vec<u8>) -> std::io::Result<()>;
    fn copy_file(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn read_path(&self, path: &Path) -> Option<Vec<u8>>;
    fn has_file(&self, path: &Path) -> bool;
    fn reset(&self) -> Result<()>;
    fn build(&self, root: &Directory) -> Result<()>;
}

impl<T: SiteBackend> SiteBackend for &T {
    fn config(&self) -> &Config {
        (*self).config()
    }
    fn add_file(&self, path: &Path, content: Vec<u8>) -> std::io::Result<()> {
        (*self).add_file(path, content)
    }
    fn copy_file(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        (*self).copy_file(to, from)
    }
    fn read_path(&self, path: &Path) -> Option<Vec<u8>> {
        (*self).read_path(path)
    }
    fn has_file(&self, path: &Path) -> bool {
        (*self).has_file(path)
    }
    fn reset(&self) -> Result<()> {
        (*self).reset()
    }
    fn build(&self, root: &Directory) -> Result<()> {
        (*self).build(root)
    }
}

#[derive(Debug)]
pub struct InMemorySite {
    config: Config,
    contents: RwLock<HashMap<PathBuf, Vec<u8>>>,
}

impl InMemorySite {
    pub fn new(config: Config) -> Self {
        InMemorySite {
            config,
            contents: RwLock::new(HashMap::new()),
        }
    }
}

impl SiteBackend for InMemorySite {
    fn config(&self) -> &Config {
        &self.config
    }

    fn add_file(&self, path: &Path, content: Vec<u8>) -> std::io::Result<()> {
        let mut contents = self.contents.write().unwrap();

        let path = path.strip_prefix(self.config.out_dir()).unwrap();

        contents.insert(path.to_owned(), content.into());
        Ok(())
    }

    fn copy_file(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        let content = fs::read(from)?;
        self.add_file(to, content)
    }

    fn read_path(&self, path: &Path) -> Option<Vec<u8>> {
        let contents = self.contents.read().unwrap();
        contents.get(path).map(|s| s.clone())
    }

    fn has_file(&self, path: &Path) -> bool {
        let contents = self.contents.read().unwrap();
        contents.contains_key(path)
    }

    fn reset(&self) -> Result<()> {
        let mut contents = self.contents.write().unwrap();
        *contents = HashMap::new();

        Ok(())
    }

    fn build(&self, root: &Directory) -> Result<()> {
        let generator = SiteGenerator::new(root, self);

        generator.run()
    }
}

pub struct DiskBackedSite {
    config: Config,
}

impl DiskBackedSite {
    pub fn new(config: Config) -> Self {
        DiskBackedSite { config }
    }

    pub fn create_dir(&self) -> Result<()> {
        fs::create_dir(&self.config.out_dir()).map_err(|e| {
            Error::io(
                e,
                format!(
                    "Could not create site directory in {}",
                    self.config.out_dir().display()
                ),
            )
        })
    }

    pub fn delete_dir(&self) -> Result<()> {
        if self.config.out_dir().exists() {
            fs::remove_dir_all(&self.config.out_dir()).map_err(|e| {
                Error::io(
                    e,
                    format!(
                        "Could not clear site directory in {}",
                        self.config.out_dir().display()
                    ),
                )
            })?
        }

        Ok(())
    }
}

impl SiteBackend for DiskBackedSite {
    fn config(&self) -> &Config {
        &self.config
    }

    fn add_file(&self, path: &Path, content: Vec<u8>) -> std::io::Result<()> {
        fs::create_dir_all(
            self.config
                .out_dir()
                .join(path.parent().expect("Path had no parent directory")),
        )?;

        fs::write(self.config.out_dir().join(path), &content)?;

        Ok(())
    }

    fn copy_file(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        fs::create_dir_all(
            self.config
                .out_dir()
                .join(to.parent().expect("Path had no parent directory")),
        )?;

        fs::copy(from, to).map(|_| ())
    }

    fn read_path(&self, path: &Path) -> Option<Vec<u8>> {
        if path.exists() {
            Some(fs::read(path).unwrap())
        } else {
            None
        }
    }

    fn has_file(&self, path: &Path) -> bool {
        path.exists()
    }

    fn reset(&self) -> Result<()> {
        self.delete_dir()?;
        self.create_dir()?;

        Ok(())
    }

    fn build(&self, root: &Directory) -> Result<()> {
        let generator = SiteGenerator::new(root, self);

        generator.run()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn you_can_add_a_file_and_read_it_back() {
        let path = Path::new("/workspace/site/index.html");
        let content = "An Content";

        let config = Config::from_yaml_str(Path::new("/workspace"), "---\ntitle: Title").unwrap();

        let site = InMemorySite::new(config);

        site.add_file(&path, content.into()).unwrap();

        let uri = Path::new("index.html");

        assert_eq!(site.read_path(uri).unwrap(), content.as_bytes());
        assert!(site.has_file(uri));
    }
}
