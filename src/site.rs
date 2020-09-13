use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::site_generator::SiteGenerator;

pub struct Site {
    out_dir: PathBuf,
}

impl Site {
    /// Create a new handle to a site output directory.
    pub fn new<P: Into<PathBuf>>(out_dir: P) -> Site {
        Site {
            out_dir: out_dir.into(),
        }
    }

    pub fn create_dir(&self) -> io::Result<()> {
        fs::create_dir(&self.out_dir)
    }

    pub fn delete_dir(&self) -> io::Result<()> {
        if self.out_dir.exists() {
            fs::remove_dir_all(&self.out_dir)?;
        }

        Ok(())
    }

    pub fn reset(&self) -> io::Result<()> {
        self.delete_dir()?;
        self.create_dir()?;

        Ok(())
    }

    fn build_from(&self, project_root: &Path) -> io::Result<()> {
        let generator = SiteGenerator::new(
            &self,
            &project_root,
            project_root.join("docs"),
            &self.out_dir,
        );

        generator.run()
    }
}
