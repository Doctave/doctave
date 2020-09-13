use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

pub struct InitCommand {
    project_root: PathBuf,
    docs_root: PathBuf,
}

impl InitCommand {
    pub fn run(project_root: PathBuf) -> io::Result<()> {
        let docs_root = project_root.join("docs");

        let cmd = InitCommand {
            project_root,
            docs_root,
        };

        cmd.create_readme()?;
        cmd.create_docs_dir()?;

        Ok(())
    }

    fn create_readme(&self) -> io::Result<()> {
        if !self.project_root.join("README.md").exists() {
            let mut file = File::create(self.project_root.join("README.md"))?;
            file.write(b"Hello, world\n============\n")?;
        }

        Ok(())
    }

    fn create_docs_dir(&self) -> io::Result<()> {
        if !self.project_root.join("docs").exists() {
            fs::create_dir(&self.docs_root)?;
        }

        Ok(())
    }
}
