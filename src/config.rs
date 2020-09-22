use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct Config {
    doctave_yaml: DoctaveYaml,
    project_root: PathBuf,
    out_dir: PathBuf,
    docs_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
struct DoctaveYaml {
    title: String,
}

impl Config {
    pub fn load(project_root: &Path) -> io::Result<Self> {
        let file = File::open(project_root.join("doctave.yaml"))?;

        let doctave_yaml: DoctaveYaml =
            serde_yaml::from_reader(file).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        Ok(Config {
            doctave_yaml,
            project_root: project_root.to_path_buf(),
            out_dir: project_root.join("site"),
            docs_dir: project_root.join("docs"),
        })
    }

    /// The title of the project
    pub fn title(&self) -> &str {
        &self.doctave_yaml.title
    }

    /// The root directory of the project - the folder containing the doctave.yaml file.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// The directory the HTML will get built into
    pub fn out_dir(&self) -> &Path {
        &self.out_dir
    }

    /// The directory that contains all the Markdown documentation
    pub fn docs_dir(&self) -> &Path {
        &self.docs_dir
    }
}

pub fn project_root() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().expect("Unable to determine current directory");

    loop {
        // If we are in the root dir, just return it
        if current_dir.join("doctave.yaml").exists() {
            return Some(current_dir);
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            return None;
        }
    }
}
