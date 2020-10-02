use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use colored::*;

use crate::{Error, Result};

pub struct InitCommand {
    project_root: PathBuf,
    docs_root: PathBuf,
}

impl InitCommand {
    pub fn run(project_root: PathBuf) -> Result<()> {
        let docs_root = project_root.join("docs");

        let cmd = InitCommand {
            project_root,
            docs_root,
        };

        println!("{}", "Doctave CLI | Init".blue().bold());
        println!("⚙️  Creating your docs...\n");

        cmd.check_for_existing_project()?;

        cmd.create_doctave_yaml()?;
        cmd.create_readme()?;
        cmd.create_docs_dir()?;

        println!(
            "\n{} Run {} to view your docs site locally.",
            "Done!".green(),
            "doctave serve".bold()
        );

        Ok(())
    }

    fn check_for_existing_project(&self) -> Result<()> {
        if self.project_root.join("doctave.yaml").exists() {
            return Err(Error::new(
                "Aborting. Found an existing doctave.yaml.\nHave you already run `doctave init`?",
            ));
        }

        Ok(())
    }

    fn create_doctave_yaml(&self) -> Result<()> {
        let mut file = File::create(self.project_root.join("doctave.yaml"))
            .map_err(|e| Error::io(e, "Could not create doctave.yaml"))?;

        file.write(b"---\ntitle: \"My Project\"\n")
            .map_err(|e| Error::io(e, "Could not write to doctave.yaml"))?;

        println!("Created doctave.yaml...");

        Ok(())
    }

    fn create_readme(&self) -> Result<()> {
        if !self.project_root.join("README.md").exists() {
            let mut file = File::create(self.project_root.join("README.md")).map_err(|e| {
                Error::io(
                    e,
                    format!(
                        "Could not create README.md in {}",
                        self.project_root.display()
                    ),
                )
            })?;

            file.write(b"Hello, world\n============\n")
                .map_err(|e| Error::io(e, "Could not write to README.md"))?;

            println!("Created README.md...");
        }

        Ok(())
    }

    fn create_docs_dir(&self) -> Result<()> {
        if !self.project_root.join("docs").exists() {
            fs::create_dir(&self.docs_root).map_err(|e| {
                Error::io(
                    e,
                    format!(
                        "Could not create docs folder in {}",
                        self.project_root.display()
                    ),
                )
            })?;

            println!("Created ./docs folder...");
        }

        Ok(())
    }
}
