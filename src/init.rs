use std::fs::{self, File};
use std::io::{self, Write};
use std::path::PathBuf;

use colored::*;

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

    fn check_for_existing_project(&self) -> io::Result<()> {
        if self.project_root.join("doctave.yaml").exists() {
            println!("Aborting. Found an existing doctave.yaml.");
            println!("Have you already run `doctave init`?");

            std::process::exit(1);
        }

        Ok(())
    }

    fn create_doctave_yaml(&self) -> io::Result<()> {
        let mut file = File::create(self.project_root.join("doctave.yaml"))?;
        file.write(b"---\ntitle: \"My Project\"\n")?;

        println!("Created doctave.yaml...");

        Ok(())
    }

    fn create_readme(&self) -> io::Result<()> {
        if !self.project_root.join("README.md").exists() {
            let mut file = File::create(self.project_root.join("README.md"))?;
            file.write(b"Hello, world\n============\n")?;

            println!("Created README.md...");
        }

        Ok(())
    }

    fn create_docs_dir(&self) -> io::Result<()> {
        if !self.project_root.join("docs").exists() {
            fs::create_dir(&self.docs_root)?;

            println!("Created ./docs folder...");
        }

        Ok(())
    }
}
