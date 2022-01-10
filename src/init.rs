use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use bunt::termcolor::{ColorChoice, StandardStream};

use crate::{Error, Result};

pub struct InitCommand {
    stdout: StandardStream,
    project_root: PathBuf,
    docs_dir: Option<PathBuf>,
}

impl InitCommand {
    pub fn run(project_root: PathBuf, colors: bool, docs_dir: Option<PathBuf>) -> Result<()> {
        let stdout = if colors {
            StandardStream::stdout(ColorChoice::Auto)
        } else {
            StandardStream::stdout(ColorChoice::Never)
        };

        let mut cmd = InitCommand {
            stdout,
            project_root,
            docs_dir,
        };

        bunt::writeln!(cmd.stdout, "{$bold}{$blue}Doctave | Init{/$}{/$}")?;
        bunt::writeln!(cmd.stdout, "Creating your docs...\n")?;

        cmd.check_for_existing_project()?;

        cmd.create_doctave_yaml()?;

        if cmd.no_existing_docs_dir() {
            cmd.create_docs_dir()?;
            cmd.create_docs_index()?;
            cmd.create_doc_examples()?;
        } else {
            let dir = cmd.docs_dir.as_deref().unwrap_or_else(|| Path::new("docs"));
            bunt::writeln!(
                cmd.stdout,
                "{$yellow}Skipping{/$} {$bold}{}{/$} directory - found existing docs...",
                dir.display()
            )?;
        }

        bunt::writeln!(
            cmd.stdout,
            "\n{$green}Done!{/$} Run {$bold}doctave serve{/$} to view your docs site locally.",
        )?;

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
        !self.doc_root().exists()
    }

    fn create_doctave_yaml(&mut self) -> Result<()> {
        let mut file = File::create(self.project_root.join("doctave.yaml"))
            .map_err(|e| Error::io(e, "Could not create doctave.yaml"))?;

        file.write(b"---\ntitle: \"My Project\"\n")
            .map_err(|e| Error::io(e, "Could not write to doctave.yaml"))?;

        if let Some(doc_root) = &self.docs_dir {
            file.write(format!("\ndocs_dir: {}\n", doc_root.display()).as_bytes())
                .map_err(|e| Error::io(e, "Could not write to doctave.yaml"))?;
        }

        bunt::writeln!(self.stdout, "Created {$bold}doctave.yaml{/$}...")?;

        Ok(())
    }

    fn create_docs_dir(&mut self) -> Result<()> {
        if self.no_existing_docs_dir() {
            fs::create_dir(&self.doc_root()).map_err(|e| {
                Error::io(
                    e,
                    format!(
                        "Could not create docs folder in {}",
                        self.project_root.display()
                    ),
                )
            })?;

            let doc_root = self.docs_dir().to_path_buf();

            bunt::writeln!(
                self.stdout,
                "Created {$bold}{}{/$} folder...",
                doc_root.display(),
            )?;
        }

        Ok(())
    }

    fn create_docs_index(&mut self) -> Result<()> {
        let path = self.doc_root().join("README.md");

        if !path.exists() {
            let mut file = File::create(path).map_err(|e| {
                Error::io(
                    e,
                    format!(
                        "Could not create README.md in {}",
                        self.doc_root().display()
                    ),
                )
            })?;

            file.write(include_str!("../templates/starter_readme.md").as_bytes())
                .map_err(|e| Error::io(e, "Could not write to README.md"))?;

            let relative_path = self.docs_dir().join("README.md");

            bunt::writeln!(
                self.stdout,
                "Created {$bold}{}{/$}...",
                relative_path.display()
            )?;
        }

        Ok(())
    }

    fn create_doc_examples(&mut self) -> Result<()> {
        let path = self.doc_root().join("examples.md");

        if !path.exists() {
            let mut file = File::create(path).map_err(|e| {
                Error::io(
                    e,
                    format!(
                        "Could not create examles.md in {}",
                        self.doc_root().display()
                    ),
                )
            })?;

            file.write(include_str!("../templates/starter_examples.md").as_bytes())
                .map_err(|e| Error::io(e, "Could not write to README.md"))?;

            let relative_path = self.docs_dir().join("examples.md");
            bunt::writeln!(
                self.stdout,
                "Created {$bold}{}{/$}...",
                relative_path.display()
            )?;
        }

        Ok(())
    }

    fn docs_dir(&self) -> &Path {
        self.docs_dir
            .as_deref()
            .unwrap_or_else(|| Path::new("docs"))
    }

    fn doc_root(&self) -> PathBuf {
        self.project_root.join(self.docs_dir())
    }
}
