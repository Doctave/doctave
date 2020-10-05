use std::fs;

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

/// A handle to the output directory where the site will be generated.
///
/// Completely agnostic about where the original Markdown files are
/// located. Only cares about the destination directory.
pub struct Site {
    config: Config,
}

impl Site {
    /// Create a new handle to a site output directory.
    pub fn new(config: Config) -> Site {
        Site { config }
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

    pub fn reset(&self) -> Result<()> {
        self.delete_dir()?;
        self.create_dir()?;

        Ok(())
    }

    pub fn build(&self) -> Result<()> {
        let generator = SiteGenerator::new(&self.config, &self);

        generator.run()
    }
}
