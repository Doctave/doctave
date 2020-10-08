use std::time::Instant;

use bunt::termcolor::{ColorChoice, StandardStream};

use crate::config::Config;
use crate::site::{BuildMode, Site};
use crate::Result;

pub struct BuildCommand {
    config: Config,
    site: Site,
}

impl BuildCommand {
    pub fn run(config: Config) -> Result<()> {
        let mut stdout = if config.color_enabled() {
            StandardStream::stdout(ColorChoice::Auto)
        } else {
            StandardStream::stdout(ColorChoice::Never)
        };

        let site = Site::new(config.clone());
        let cmd = BuildCommand { config, site };

        let target_dir = &cmd.config.out_dir();

        bunt::writeln!(stdout, "{$bold}{$blue}Doctave | Build{/$}{/$}")?;

        if let BuildMode::Release = cmd.config.build_mode() {
            bunt::writeln!(
                stdout,
                "Building site into {$bold}{}{/$} in {$bold}release mode{/$}\n",
                target_dir.display(),
            )?;
        } else {
            bunt::writeln!(
                stdout,
                "Building site into {$bold}{}{/$}\n",
                target_dir.display()
            )?;
        }

        let start = Instant::now();
        let result = cmd.site.build();
        let duration = start.elapsed();

        if result.is_ok() {
            bunt::writeln!(stdout, "Site built in {$bold}{:?}{/$}\n", duration)?;
        }

        result
    }
}
