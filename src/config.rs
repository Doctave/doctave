use std::fs;
use std::path::{Path, PathBuf};

use colorsys::prelude::*;
use colorsys::Rgb;
use http::Uri;
use serde::Deserialize;

use crate::navigation::Link;
use crate::site::BuildMode;
use crate::{Error, Result};

#[derive(Debug, Clone, Deserialize)]
struct DoctaveYaml {
    title: String,
    port: Option<u32>,
    colors: Option<ColorsYaml>,
    logo: Option<PathBuf>,
    navigation: Option<Vec<Navigation>>,
    base_path: Option<String>,
    docs_dir: Option<String>,
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

    /// Runs checks that validate the values of provided in the Yaml file
    fn validate(&mut self, project_root: &Path) -> Result<()> {
        // Get the root doc path
        // We don't validate if it exists or not because the rest
        // of the application is responsible for assuming that "docs"
        // is the default, and checking whether or not the directory
        // exists.
        let docs_dir_path = self.docs_dir(project_root);

        // Validate color
        if let Some(color) = &self.colors.as_ref().and_then(|c| c.main.as_ref()) {
            Rgb::from_hex_str(color).map_err(|_e| {
                Error::new(format!(
                    "Invalid HEX color provided for \
                    colors.main in doctave.yaml.\nFound '{}'",
                    &self.colors.as_ref().and_then(|c| c.main.as_ref()).unwrap()
                ))
            })?;
        }

        // Validate logo exists
        if let Some(p) = &self.logo {
            let location = docs_dir_path.join("_include").join(p);
            if !location.exists() {
                return Err(Error::new(format!(
                    "Could not find logo specified in doctave.yaml at {}.\n\
                     The logo path should be relative to the _include directory.",
                    location.display()
                )));
            }
        }

        // Validate navigation paths exist
        // Validate navigation wildcards recursively
        fn validate_level(
            nav: &Navigation,
            config: &DoctaveYaml,
            project_root: &Path,
        ) -> Result<()> {
            if !project_root.join(&nav.path).exists() {
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
                            validate_level(&nav, config, project_root)?;
                        }
                    }
                }
            }

            Ok(())
        }

        if let Some(navs) = &self.navigation {
            for nav in navs {
                validate_level(nav, &self, &project_root)?;
            }
        }

        // Validate base path
        if let Some(path) = &mut self.base_path {
            let uri: Uri = path.parse().map_err(|_| {
                Error::new(format!(
                    "base_path was not valid absolute URI path. Got `{}`",
                    path
                ))
            })?;

            if !uri.path().starts_with("/") {
                return Err(Error::new(format!(
                    "Base path must be an absolute path. Got `{}`.",
                    path
                )));
            }

            if !path.ends_with("/") {
                path.push('/');
            }
        }

        Ok(())
    }

    fn docs_dir(&self, project_root: &Path) -> PathBuf {
        let to_join = match &self.docs_dir {
            Some(docs_dir) => docs_dir.clone(),
            None => "docs".to_string(),
        };

        let doc_root_path = project_root.join(to_join);
        doc_root_path
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

static DEFAULT_THEME_COLOR: &str = "#445282";

#[derive(Debug, Clone)]
struct Colors {
    main: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct ColorsYaml {
    main: Option<String>,
}

impl From<ColorsYaml> for Colors {
    fn from(other: ColorsYaml) -> Self {
        Colors {
            main: other.main.unwrap_or(DEFAULT_THEME_COLOR.to_owned()),
        }
    }
}

impl Default for Colors {
    fn default() -> Self {
        Colors {
            main: DEFAULT_THEME_COLOR.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NavRule {
    File(PathBuf),
    Dir(PathBuf, Option<DirIncludeRule>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirIncludeRule {
    WildCard,
    Explicit(Vec<NavRule>),
}

impl NavRule {
    fn from_yaml_input(input: Vec<Navigation>) -> Vec<NavRule> {
        let mut rules = vec![];

        for item in input {
            if item.path.is_file() {
                rules.push(NavRule::File(item.path.clone()));
            } else if item.path.is_dir() {
                let dir_rules = Self::build_directory_rules(&item);
                rules.push(dir_rules);
            }
        }

        rules
    }

    fn build_directory_rules(dir: &Navigation) -> NavRule {
        match &dir.children {
            None => NavRule::Dir(dir.path.clone(), None),
            Some(NavChildren::WildCard(_)) => {
                NavRule::Dir(dir.path.clone(), Some(DirIncludeRule::WildCard))
            }
            Some(NavChildren::List(paths)) => NavRule::Dir(
                dir.path.clone(),
                Some(DirIncludeRule::Explicit(
                    paths
                        .iter()
                        .map(|p| {
                            if p.path.is_file() {
                                NavRule::File(p.path.clone())
                            } else {
                                Self::build_directory_rules(p)
                            }
                        })
                        .collect::<Vec<_>>(),
                )),
            ),
        }
    }

    pub fn is_default_readme_rule(&self, root_dir: &Path, docs_dir: &Path) -> bool {
        let my_path = match self {
            NavRule::File(path) => path,
            NavRule::Dir(_, _) => return false,
        };

        root_dir.join(my_path) == docs_dir.join("README.md")
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    color: bool,
    skip_checks: bool,
    project_root: PathBuf,
    out_dir: PathBuf,
    docs_dir: PathBuf,
    base_path: String,
    title: String,
    colors: Colors,
    logo: Option<String>,
    navigation: Option<Vec<NavRule>>,
    port: u32,
    build_mode: BuildMode,
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

        doctave_yaml.validate(project_root)?;

        let config = Config {
            color: true,
            skip_checks: false,
            project_root: project_root.to_path_buf(),
            out_dir: project_root.join("site"),
            docs_dir: doctave_yaml.docs_dir(project_root),
            base_path: doctave_yaml.base_path.unwrap_or(String::from("/")),
            title: doctave_yaml.title,
            colors: doctave_yaml
                .colors
                .map(|c| c.into())
                .unwrap_or(Colors::default()),
            logo: doctave_yaml
                .logo
                .map(|p| Link::path_to_uri_with_extension(&p))
                .map(|p| p.as_str().trim_start_matches("/").to_owned()),
            navigation: doctave_yaml.navigation.map(|n| NavRule::from_yaml_input(n)),
            port: doctave_yaml.port.unwrap_or_else(|| 4001),
            build_mode: BuildMode::Dev,
        };

        Ok(config)
    }

    /// The title of the project
    pub fn title(&self) -> &str {
        &self.title
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

    /// The directory that contains all the Markdown documentation
    pub fn base_path(&self) -> &str {
        &self.base_path
    }

    /// Rules that set the site navigation structure
    pub fn navigation(&self) -> Option<&[NavRule]> {
        self.navigation.as_deref()
    }

    /// Port to serve the development server on
    pub fn port(&self) -> u32 {
        self.port
    }

    pub fn color_enabled(&self) -> bool {
        self.color
    }

    pub fn skip_checks(&self) -> bool {
        self.skip_checks
    }

    pub fn disable_colors(&mut self) {
        self.color = false
    }

    pub fn disable_checks(&mut self) {
        self.skip_checks = true
    }

    pub fn build_mode(&self) -> BuildMode {
        self.build_mode
    }

    pub fn set_build_mode(&mut self, mode: BuildMode) {
        self.build_mode = mode;
    }

    /// The main theme color. Other shades are computed based off of this
    /// color.
    ///
    /// Must be a valid HEX color.
    pub fn main_color(&self) -> Rgb {
        let color = &self.colors.main;

        // This was already validated
        Rgb::from_hex_str(color).unwrap()
    }

    /// A lighter version of the main color, meant to be used in _dark_ mode.
    pub fn main_color_dark(&self) -> Rgb {
        let mut color = self.main_color();
        color.lighten(25.0);
        color
    }

    /// URI path to a logo that will show up at the top left next to the title
    pub fn logo(&self) -> Option<&str> {
        self.logo.as_deref()
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

        let error = Config::from_yaml_str(Path::new(""), yaml).unwrap_err();

        assert!(
            format!("{}", error)
                .contains("Invalid HEX color provided for colors.main in doctave.yaml"),
            "Error message was: {}",
            error
        );
        assert!(
            format!("{}", error).contains("Found 'not-a-color'"),
            "Error message was: {}",
            error
        );
    }

    #[test]
    fn validate_logo() {
        let yaml = indoc! {"
            ---
            title: The Title
            logo: i-do-not-exist.png
        "};

        let error = Config::from_yaml_str(Path::new(""), yaml).unwrap_err();

        assert!(
            format!("{}", error).contains("Could not find logo specified in doctave.yaml"),
            "Error message was: {}",
            error
        );
    }

    #[test]
    fn validate_base_path() {
        let yaml = indoc! {"
            ---
            title: The Title
            base_path: not/absolute
        "};

        let error = Config::from_yaml_str(Path::new(""), yaml).unwrap_err();

        println!("{:?}", error);

        assert!(
            format!("{}", error)
                .contains("base_path was not valid absolute URI path. Got `not/absolute`"),
            "Got incorrect error message: {}",
            error
        );
    }

    #[test]
    fn validate_base_path_ends_with_slash() {
        let yaml = indoc! {"
            ---
            title: The Title
            base_path: /docs
        "};

        let config = Config::from_yaml_str(Path::new(""), yaml).unwrap();

        assert_eq!(config.base_path(), "/docs/");
    }

    #[test]
    fn validate_default_base_path() {
        let yaml = indoc! {"
            ---
            title: The Title
        "};

        let config = Config::from_yaml_str(Path::new(""), yaml).unwrap();

        assert_eq!(config.base_path(), "/");
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

        let error = Config::from_yaml_str(Path::new(""), yaml).unwrap_err();

        assert!(
            format!("{}", error).contains(
                "Invalid pattern for navigation children. \
                Found 'not-wildcard', expected \"*\" or a list of child pages"
            ),
            "Error message was: {}",
            error
        );
    }

    #[test]
    fn convert_navigation_input_to_rules_file() {
        let input = vec![Navigation {
            path: PathBuf::from("docs").join("README.md"),
            children: None,
        }];

        assert_eq!(
            NavRule::from_yaml_input(input),
            vec![NavRule::File(PathBuf::from("docs").join("README.md"))]
        );
    }

    #[test]
    fn convert_navigation_input_to_rules_directory_no_children() {
        let input = vec![Navigation {
            path: PathBuf::from("docs").join("features"), // TODO: Make not rely on our docs
            children: None,
        }];

        assert_eq!(
            NavRule::from_yaml_input(input),
            vec![NavRule::Dir(PathBuf::from("docs").join("features"), None)]
        );
    }

    #[test]
    fn convert_navigation_input_to_rules_directory_wildcard_children() {
        let input = vec![Navigation {
            path: PathBuf::from("docs").join("features"), // TODO: Make not rely on our docs
            children: Some(NavChildren::WildCard(String::from("*"))),
        }];

        assert_eq!(
            NavRule::from_yaml_input(input),
            vec![NavRule::Dir(
                PathBuf::from("docs").join("features"),
                Some(DirIncludeRule::WildCard)
            )]
        );
    }

    #[test]
    fn convert_navigation_input_to_rules_directory_explicit_children() {
        let input = vec![Navigation {
            path: PathBuf::from("docs").join("features"), // TODO: Make not rely on our docs
            children: Some(NavChildren::List(vec![Navigation {
                path: PathBuf::from("docs").join("features").join("markdown.md"),
                children: None,
            }])),
        }];

        assert_eq!(
            NavRule::from_yaml_input(input),
            vec![NavRule::Dir(
                PathBuf::from("docs").join("features"),
                Some(DirIncludeRule::Explicit(vec![NavRule::File(
                    PathBuf::from("docs").join("features").join("markdown.md")
                )]))
            )]
        );
    }
}
