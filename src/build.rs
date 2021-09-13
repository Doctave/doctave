use std::time::Instant;

use bunt::termcolor::{ColorChoice, StandardStream};

use crate::config::Config;
use crate::site::{BuildMode, DiskBackedSite, Site};
use crate::Result;

pub struct BuildCommand {}

impl BuildCommand {
    pub fn run(config: Config) -> Result<()> {
        let mut stdout = if config.color_enabled() {
            StandardStream::stdout(ColorChoice::Auto)
        } else {
            StandardStream::stdout(ColorChoice::Never)
        };

        let site = DiskBackedSite::new(config.clone());

        let target_dir = config.out_dir();

        bunt::writeln!(stdout, "{$bold}{$blue}Doctave | Build{/$}{/$}")?;

        if let BuildMode::Release = config.build_mode() {
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
        let result = site.build();
        let duration = start.elapsed();

        if result.is_ok() {
            bunt::writeln!(stdout, "Site built in {$bold}{:?}{/$}\n", duration)?;
        }

        result
    }
}
