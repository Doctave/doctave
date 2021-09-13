use std::collections::HashMap;
use std::sync::RwLock;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::site_generator::SiteGenerator;
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

pub trait Site: Send + Sync {
    fn config(&self) -> &Config;
    fn add_file(&self, path: &Path, content: Vec<u8>) -> std::io::Result<()>;
    fn copy_file(&self, from: &Path, to: &Path) -> std::io::Result<()>;
    fn read_path(&self, path: &Path) -> Option<Vec<u8>>;
    fn has_file(&self, path: &Path) -> bool;
    fn reset(&self) -> Result<()>;
    fn build(&self) -> Result<()>;
}

impl<T: Site> Site for &T {
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
}

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

impl Site for InMemorySite {
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

    fn build(&self) -> Result<()> {
        let generator = SiteGenerator::new(self);

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

impl Site for DiskBackedSite {
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

    fn build(&self) -> Result<()> {
        let generator = SiteGenerator::new(self);

        generator.run()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn you_can_add_a_file_and_read_it_back() {
        let path = Path::new("index.html");
        let content = "An Content";

        let config = Config::from_yaml_str(Path::new(""), "---\ntitle: Title").unwrap();
        let site = InMemorySite::new(config);

        site.add_file(&path, content.into()).unwrap();

        assert_eq!(site.read_path(path).unwrap(), content.as_bytes());
        assert!(site.has_file(&path));
    }
}
