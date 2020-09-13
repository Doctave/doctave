use std::io;
use std::path::PathBuf;

pub struct ServeCommand {
    project_root: PathBuf,
    docs_root: PathBuf,
    out: PathBuf,
}

impl ServeCommand {
    pub fn run(root: PathBuf) -> io::Result<()> {
        let cmd = ServeCommand {
            project_root: root.clone(),
            docs_root: root.join("docs"),
            out: root.join("site"),
        };

        Ok(())
    }
}
