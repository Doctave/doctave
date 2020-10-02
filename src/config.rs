use std::fs::File;
use std::path::{Path, PathBuf};

use colorsys::prelude::*;
use colorsys::Rgb;
use serde::Deserialize;

use crate::{Error, Result};

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
    colors: Option<Colors>,
    logo: Option<PathBuf>,
}

static DEFAULT_THEME_COLOR: &'static str = "#445282";

#[derive(Debug, Clone, Deserialize, Default)]
struct Colors {
    main: Option<String>,
}

impl Config {
    pub fn load(project_root: &Path) -> Result<Self> {
        let file = File::open(project_root.join("doctave.yaml"))
            .map_err(|_| Error::new("Could not open doctave.yaml file"))?;

        let mut doctave_yaml: DoctaveYaml = serde_yaml::from_reader(file)
            .map_err(|e| Error::yaml(e, "Could not parse doctave.yaml"))?;

        if doctave_yaml.colors.is_none() {
            doctave_yaml.colors = Some(Colors::default());
        }

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

    /// The main theme color. Other shades are computed based off of this
    /// color.
    ///
    /// Must be a valid HEX color.
    pub fn main_color(&self) -> Rgb {
        let color = self
            .doctave_yaml
            .colors
            .as_ref()
            .unwrap()
            .main
            .as_deref()
            .unwrap_or(DEFAULT_THEME_COLOR);

        Rgb::from_hex_str(color).unwrap_or_else(|_| {
            println!(
                "Could not parse color code \"{}\" from doctave.yaml",
                self.doctave_yaml
                    .colors
                    .as_ref()
                    .unwrap()
                    .main
                    .as_deref()
                    .unwrap()
            );
            println!("Colors must be specified as HEX values. For example: #5F658A");

            std::process::exit(1);
        })
    }

    /// A lighter version of the main color, meant to be used in _dark_ mode.
    pub fn main_color_dark(&self) -> Rgb {
        let mut color = self.main_color();
        color.lighten(25.0);
        color
    }

    /// Path to a logo that will show up at the top left next to the title
    pub fn logo(&self) -> Option<PathBuf> {
        if let Some(p) = &self.doctave_yaml.logo {
            let location = self.docs_dir.join("_assets").join(p);
            if !location.exists() {
                println!(
                    "Could not find logo specified in doctave.yaml at {}",
                    p.display()
                );

                std::process::exit(1);
            } else {
                Some(PathBuf::from("/assets").join(p))
            }
        } else {
            None
        }
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
