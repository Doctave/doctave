use std::time::Instant;

use colored::*;

use crate::config::Config;
use crate::site::{BuildMode, Site};
use crate::Result;

pub struct BuildCommand {
    config: Config,
    site: Site,
}

impl BuildCommand {
    pub fn run(config: Config) -> Result<()> {
        let site = Site::new(config.clone());
        let cmd = BuildCommand { config, site };

        let target_dir = &cmd.config.out_dir();

        println!("{}", "Doctave CLI | Build".blue().bold());

        if let BuildMode::Release = cmd.config.build_mode() {
            println!(
                "🏗️  Building site into {} in {}\n",
                format!("{}", target_dir.display()).bold(),
                "release mode".bold(),
            );
        } else {
            println!(
                "🏗️  Building site into {}\n",
                format!("{}", target_dir.display()).bold()
            );
        }

        let start = Instant::now();
        let result = cmd.site.build();
        let duration = start.elapsed();

        if result.is_ok() {
            println!("Site built in {}\n", format!("{:?}", duration).bold());
        }

        result
    }
}
