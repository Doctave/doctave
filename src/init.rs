use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

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

        if cmd.no_existing_docs_dir() {
            cmd.create_docs_dir()?;
            cmd.create_docs_index()?;
            cmd.create_doc_examples()?;
        } else {
            println!(
                "{} {} directory - found existing docs...",
                "Skipping".yellow(),
                "docs".bold(),
            );
        }

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
                "Aborting. Found an existing doctave.yaml.\nHave you already run doctave init?",
            ));
        }

        Ok(())
    }

    fn no_existing_docs_dir(&self) -> bool {
        !self.project_root.join("docs").exists()
    }

    fn create_doctave_yaml(&self) -> Result<()> {
        let mut file = File::create(self.project_root.join("doctave.yaml"))
            .map_err(|e| Error::io(e, "Could not create doctave.yaml"))?;

        file.write(b"---\ntitle: \"My Project\"\n")
            .map_err(|e| Error::io(e, "Could not write to doctave.yaml"))?;

        println!("Created {}...", "doctave.yaml".bold());

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

            println!("Created {} folder...", "docs".bold());
        }

        Ok(())
    }

    fn create_docs_index(&self) -> Result<()> {
        let path = self.project_root.join("docs").join("README.md");

        if !path.exists() {
            let mut file = File::create(path).map_err(|e| {
                Error::io(
                    e,
                    format!(
                        "Could not create README.md in {}",
                        self.project_root.join("docs").display()
                    ),
                )
            })?;

            file.write(include_str!("../templates/starter_readme.md").as_bytes())
                .map_err(|e| Error::io(e, "Could not write to README.md"))?;

            println!(
                "Created {}...",
                Path::new("docs")
                    .join("README.md")
                    .display()
                    .to_string()
                    .bold()
            );
        }

        Ok(())
    }

    fn create_doc_examples(&self) -> Result<()> {
        let path = self.project_root.join("docs").join("examples.md");

        if !path.exists() {
            let mut file = File::create(path).map_err(|e| {
                Error::io(
                    e,
                    format!(
                        "Could not create examles.md in {}",
                        self.project_root.join("docs").display()
                    ),
                )
            })?;

            file.write(include_str!("../templates/starter_examples.md").as_bytes())
                .map_err(|e| Error::io(e, "Could not write to README.md"))?;

            println!(
                "Created {}...",
                Path::new("docs")
                    .join("examples.md")
                    .display()
                    .to_string()
                    .bold()
            );
        }

        Ok(())
    }
}
