use std::collections::{BTreeMap, HashMap};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use walkdir::WalkDir;

use crate::broken_links_checker;
use crate::config::Config;
use crate::docs_finder;
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
    pub backend: B,
    pub config: Config,
}

impl Site<InMemorySite> {
    pub fn in_memory(config: Config) -> Site<InMemorySite> {
        Site {
            backend: InMemorySite::new(config.clone()),
            config,
        }
    }

    #[cfg(test)]
    /// Don't load the site from memory - instead provide the loaded directory
    /// state manually. Used only in testing.
    pub fn with_root(root: Directory, config: Config) -> Site<InMemorySite> {
        Site {
            backend: InMemorySite::with_root(root, config.clone()),
            config,
        }
    }
}

impl Site<DiskBackedSite> {
    pub fn disk_backed(config: Config) -> Site<DiskBackedSite> {
        Site {
            backend: DiskBackedSite::new(config.clone()),
            config,
        }
    }
}

impl<B: SiteBackend> Site<B> {
    pub fn root(&self) -> Directory {
        self.backend.root()
    }

    pub fn reset(&self) -> Result<()> {
        self.backend.reset()
    }

    pub fn build(&self) -> Result<()> {
        self.backend.build()
    }

    pub fn rebuild(&self) -> Result<()> {
        self.backend.reset()?;
        self.backend.build()
    }

    pub fn check_dead_links(&self) -> Result<()> {
        broken_links_checker::run(&self)
    }
}

pub trait SiteBackend: Send + Sync {
    fn root(&self) -> Directory;
    fn config(&self) -> &Config;
    /// Adds the rendered content for a given path
    fn add_file(&self, path: &Path, content: Vec<u8>) -> std::io::Result<()>;
    fn copy_file(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    /// Reads the rendered output of the specified path
    fn read_path(&self, path: &Path) -> Option<Vec<u8>>;
    /// Says if we have rendered the specified file
    fn has_file(&self, path: &Path) -> bool;
    /// Clears the rendered output, and reloads the documentation from disk into memory
    fn reset(&self) -> Result<()>;
    /// Renders the loaded documentation into memory
    fn build(&self) -> Result<()>;
    fn list_files(&self) -> Vec<PathBuf>;
}

impl<T: SiteBackend> SiteBackend for &T {
    fn root(&self) -> Directory {
        (*self).root()
    }
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
    fn build(&self) -> Result<()> {
        (*self).build()
    }
    fn list_files(&self) -> Vec<PathBuf> {
        (*self).list_files()
    }
}

#[derive(Debug)]
pub struct InMemorySite {
    config: Config,
    content: RwLock<InMemoryContent>,
}

#[derive(Debug)]
struct InMemoryContent {
    pub root: Directory,
    pub rendered: HashMap<PathBuf, Vec<u8>>,
}

impl InMemorySite {
    pub fn new(config: Config) -> Self {
        InMemorySite {
            content: RwLock::new(InMemoryContent {
                root: docs_finder::find(&config),
                rendered: HashMap::new(),
            }),
            config,
        }
    }

    #[cfg(test)]
    pub fn with_root(root: Directory, config: Config) -> Self {
        InMemorySite {
            content: RwLock::new(InMemoryContent {
                root,
                rendered: HashMap::new(),
            }),
            config,
        }
    }
}

impl SiteBackend for InMemorySite {
    fn root(&self) -> Directory {
        let content = self.content.read().unwrap();
        content.root.clone()
    }

    fn config(&self) -> &Config {
        &self.config
    }

    fn add_file(&self, path: &Path, html: Vec<u8>) -> std::io::Result<()> {
        let mut content = self.content.write().unwrap();

        let path = path.strip_prefix(self.config.out_dir()).unwrap();

        content.rendered.insert(path.to_owned(), html.into());
        Ok(())
    }

    fn copy_file(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        let content = fs::read(from)?;
        self.add_file(to, content)
    }

    fn read_path(&self, path: &Path) -> Option<Vec<u8>> {
        let content = self.content.read().unwrap();
        content.rendered.get(path).map(|s| s.clone())
    }

    fn has_file(&self, path: &Path) -> bool {
        let content = self.content.read().unwrap();
        content.rendered.contains_key(path)
    }

    fn reset(&self) -> Result<()> {
        let mut content = self.content.write().unwrap();
        content.rendered = HashMap::new();
        content.root = docs_finder::find(&self.config);

        Ok(())
    }

    fn build(&self) -> Result<()> {
        let generator = SiteGenerator::new(self);

        generator.run()
    }

    fn list_files(&self) -> Vec<PathBuf> {
        let content = self.content.read().unwrap();

        content
            .rendered
            .keys()
            .map(|p| p.to_owned())
            .collect::<Vec<_>>()
    }
}

pub struct DiskBackedSite {
    config: Config,
    root: Directory,
}

impl DiskBackedSite {
    pub fn new(config: Config) -> Self {
        DiskBackedSite {
            root: docs_finder::find(&config),
            config,
        }
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
    fn root(&self) -> Directory {
        self.root.clone()
    }

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
        if self.config.out_dir().join(path).exists() {
            Some(fs::read(self.config.out_dir().join(path)).unwrap())
        } else {
            None
        }
    }

    fn has_file(&self, path: &Path) -> bool {
        self.config.out_dir().join(path).exists()
    }

    fn reset(&self) -> Result<()> {
        self.delete_dir()?;
        self.create_dir()?;

        Ok(())
    }

    fn build(&self) -> Result<()> {
        let generator = SiteGenerator::new(self);

        generator.run()
    }

    fn list_files(&self) -> Vec<PathBuf> {
        walkdir::WalkDir::new(self.config.out_dir())
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_owned())
            .collect::<Vec<_>>()
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
