use std::fs;
use std::path::{Path, PathBuf};

use colorsys::prelude::*;
use colorsys::Rgb;
use serde::Deserialize;

use crate::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
struct DoctaveYaml {
    title: String,
    colors: Option<Colors>,
    logo: Option<PathBuf>,
    navigation: Option<Vec<Navigation>>,
}

impl DoctaveYaml {
    fn find(root: &Path) -> Option<PathBuf> {
        if root.join("doctave.yaml").exists() {
            Some(root.join("doctave.yaml"))
        } else if root.join("doctave.yml").exists() {
            Some(root.join("doctave.yml"))
        } else {
            None
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
pub struct Navigation {
    pub path: PathBuf,
    pub children: Option<NavChildren>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum NavChildren {
    WildCard(String),
    List(Vec<Navigation>),
}

static DEFAULT_THEME_COLOR: &'static str = "#445282";

#[derive(Debug, Clone, Deserialize, Default)]
struct Colors {
    main: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    doctave_yaml: DoctaveYaml,
    project_root: PathBuf,
    out_dir: PathBuf,
    docs_dir: PathBuf,
}

pub enum NavRule {
    File(PathBuf),
    Dir(PathBuf, DirIncludeRule),
}

pub enum DirIncludeRule {
    WildCard,
    Explicit(Vec<NavRule>)
}

impl Config {
    pub fn load(project_root: &Path) -> Result<Self> {
        let path = DoctaveYaml::find(&project_root)
            .ok_or(Error::new("Could not find doctave.yaml in project"))?;

        let yaml =
            fs::read_to_string(path).map_err(|_| Error::new("Could not read doctave.yaml file"))?;

        Config::from_yaml_str(project_root, &yaml)
    }

    pub fn from_yaml_str(project_root: &Path, yaml: &str) -> Result<Self> {
        let mut doctave_yaml: DoctaveYaml = serde_yaml::from_str(yaml)
            .map_err(|e| Error::yaml(e, "Could not parse doctave.yaml"))?;

        if doctave_yaml.colors.is_none() {
            doctave_yaml.colors = Some(Colors::default());
        }

        let config = Config {
            doctave_yaml,
            project_root: project_root.to_path_buf(),
            out_dir: project_root.join("site"),
            docs_dir: project_root.join("docs"),
        };

        Ok(config)
    }

    /// Runs checks that validate the values of provided in the config.
    pub fn validate(&self) -> Result<()> {
        // Validate color
        if let Some(colors) = self.doctave_yaml.colors.as_ref() {
            if let Some(main_color) = &colors.main {
                Rgb::from_hex_str(&main_color).map_err(|_e| {
                    Error::new(format!(
                        "Invalid HEX color provided for \
                        colors.main in doctave.yaml.\nFound '{}'",
                        main_color
                    ))
                })?;
            }
        }

        // Validate logo exists
        if let Some(p) = &self.doctave_yaml.logo {
            let location = self.docs_dir.join("_assets").join(p);
            if !location.exists() {
                return Err(Error::new(format!(
                    "Could not find logo specified in doctave.yaml at {}",
                    p.display()
                )));
            }
        }

        // Validate navigation paths exist
        // Validate navigation wildcards recursively
        fn validate_level(nav: &Navigation, config: &Config) -> Result<()> {
            if !config.project_root().join(&nav.path).exists() {
                return Err(Error::new(format!(
                    "Could not find file specified in navigation at {}",
                    nav.path.display()
                )));
            }

            if let Some(children) = &nav.children {
                match children {
                    NavChildren::WildCard(pattern) => {
                        if pattern != "*" {
                            return Err(Error::new(format!(
                                "Invalid pattern for navigation children. \
                                    Found '{}', expected \"*\" or a list of child pages",
                                pattern
                            )));
                        }
                    }
                    NavChildren::List(navs) => {
                        for nav in navs {
                            validate_level(&nav, config)?;
                        }
                    }
                }
            }

            Ok(())
        }

        if let Some(navs) = &self.doctave_yaml.navigation {
            for nav in navs {
                validate_level(nav, &self)?;
            }
        }

        Ok(())
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

    pub fn navigation(&self) -> Option<&[Navigation]> {
        self.doctave_yaml.navigation.as_deref()
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

        // This was already validated
        Rgb::from_hex_str(color).unwrap()
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
        if current_dir.join("doctave.yaml").exists() || current_dir.join("doctave.yml").exists() {
            return Some(current_dir);
        }

        if let Some(parent) = current_dir.parent() {
            current_dir = parent.to_path_buf();
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    extern crate indoc;

    #[test]
    fn validate_colors() {
        let yaml = indoc! {"
            ---
            title: The Title
            colors:
               main: not-a-color
        "};

        let config = Config::from_yaml_str(Path::new(""), yaml).unwrap();
        let error = config.validate().unwrap_err();

        assert!(
            format!("{}", error)
                .contains("Invalid HEX color provided for colors.main in doctave.yaml"),
            format!("Error message was: {}", error)
        );
        assert!(
            format!("{}", error).contains("Found 'not-a-color'"),
            format!("Error message was: {}", error)
        );
    }

    #[test]
    fn validate_logo() {
        let yaml = indoc! {"
            ---
            title: The Title
            logo: i-do-not-exist.png
        "};

        let config = Config::from_yaml_str(Path::new(""), yaml).unwrap();
        let error = config.validate().unwrap_err();

        assert!(
            format!("{}", error)
                .contains("Could not find logo specified in doctave.yaml at i-do-not-exist.png"),
            format!("Error message was: {}", error)
        );
    }

    #[test]
    fn validate_navigation_wildcard() {
        let yaml = indoc! {"
            ---
            title: The Title
            navigation:
              - path: docs/tutorial.md
                children: not-wildcard
        "};

        let config = Config::from_yaml_str(Path::new(""), yaml).unwrap();
        let error = config.validate().unwrap_err();

        assert!(
            format!("{}", error).contains(
                "Invalid pattern for navigation children. \
                Found 'not-wildcard', expected \"*\" or a list of child pages"
            ),
            format!("Error message was: {}", error)
        );
    }
}
