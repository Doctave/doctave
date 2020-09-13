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

        cmd.reset_site_dir()?;
        cmd.build_site()?;

        Ok(())
    }

    fn reset_site_dir(&self) -> io::Result<()> {
        self.site.reset()
    }

    fn build_site(&self) -> io::Result<()> {
        self.site.build_from(&self.project_root)
    }
}
