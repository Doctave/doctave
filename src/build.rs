use std::env::current_dir;
use std::io;
use std::path::PathBuf;
use std::time::Instant;

use colored::*;

use crate::site::Site;

pub struct BuildCommand {
    project_root: PathBuf,
    site: Site,
}

impl BuildCommand {
    pub fn run(root: PathBuf) -> io::Result<()> {
        let cmd = BuildCommand {
            project_root: root.clone(),
            site: Site::in_dir(root.join("site")),
        };

        println!("{}", "Doctave CLI | Serve".blue().bold());
        println!(
            "🏗️  Building site into {}\n",
            format!(
                "{}",
                &cmd.site
                    .out_dir()
                    .strip_prefix(current_dir()?)
                    .map(|d| d.display())
                    .unwrap_or(cmd.site.out_dir().display())
            )
            .bold()
        );

        let start = Instant::now();
        let result = cmd.site.build_from(&cmd.project_root);
        let duration = start.elapsed();

        if result.is_ok() {
            println!("Site built in {}\n", format!("{:?}", duration).bold());
        }

        result
    }
}
