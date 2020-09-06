use std::path::PathBuf;
use std::io::{self, Write};
use std::fs::{self, File};

pub struct InitCommand {
    root: PathBuf,
}

impl InitCommand {
    pub fn run(root: PathBuf) -> io::Result<()> {
        let cmd = InitCommand { root };

        cmd.create_readme()?;
        cmd.create_docs_dir()?;

        Ok(())
    }

    fn create_readme(&self) -> io::Result<()> {
        if !self.root.join("README.md").exists() {
            let mut file = File::create(self.root.join("README.md"))?;
            file.write(b"Hello, world\n============\n")?;
        }

        Ok(())
    }

    fn create_docs_dir(&self) -> io::Result<()> {
        if !self.root.join("docs").exists() {
            fs::create_dir(self.root.join("docs"))?;
        }

        Ok(())
    }
}

