use std::env::current_dir;
use std::io;
use std::time::Instant;

use colored::*;

use crate::config::Config;
use crate::site::Site;

pub struct BuildCommand {
    config: Config,
    site: Site,
}

impl BuildCommand {
    pub fn run(config: Config) -> io::Result<()> {
        let site = Site::new(config.clone());
        let cmd = BuildCommand { config, site };

        println!("{}", "Doctave CLI | Serve".blue().bold());
        println!(
            "🏗️  Building site into {}\n",
            format!(
                "{}",
                &cmd.config
                    .out_dir()
                    .strip_prefix(current_dir()?)
                    .map(|d| d.display())
                    .unwrap_or(cmd.config.out_dir().display())
            )
            .bold()
        );

        let start = Instant::now();
        let result = cmd.site.build();
        let duration = start.elapsed();

        if result.is_ok() {
            println!("Site built in {}\n", format!("{:?}", duration).bold());
        }

        result
    }
}
