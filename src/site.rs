use std::fs;
use std::io;

use crate::config::Config;
use crate::site_generator::SiteGenerator;

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

    pub fn create_dir(&self) -> io::Result<()> {
        fs::create_dir(&self.config.out_dir())
    }

    pub fn delete_dir(&self) -> io::Result<()> {
        if self.config.out_dir().exists() {
            fs::remove_dir_all(&self.config.out_dir())?;
        }

        Ok(())
    }

    pub fn reset(&self) -> io::Result<()> {
        self.delete_dir()?;
        self.create_dir()?;

        Ok(())
    }

    pub fn build(&self) -> io::Result<()> {
        let generator = SiteGenerator::new(&self.config, &self);

        generator.run()
    }
}
