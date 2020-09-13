use std::io;
use std::path::PathBuf;

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

        cmd.site.build_from(&cmd.project_root)
    }
}
