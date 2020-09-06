use std::env::current_exe;
use std::fs::{remove_dir_all, create_dir_all};
use std::io;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

static TEST_WORKDIRS: &'static str = "_test_area";

pub fn test_dir<F, P: AsRef<Path>>(name: P, lambda: F)
where
    F: FnOnce(&TestDir),
{
    let dir = TestDir::create(name).expect("Could not create test dir");

    (lambda)(&dir);
}

pub struct TestDir {
    root: PathBuf,
    dir: PathBuf,
}

impl TestDir {
    pub fn create<P: AsRef<Path>>(name: P) -> io::Result<Self> {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(TEST_WORKDIRS)
            .join(name);
        let root = current_exe()
            .unwrap()
            .parent()
            .expect("executable's directory")
            .to_path_buf();

        println!("{:?}", dir);

        if dir.exists() {
            remove_dir_all(&dir)?;
        }
        create_dir_all(&dir)?;

        Ok(TestDir { dir, root })
    }

    pub fn cmd<I, S>(&self, args: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new(self.binary())
            .args(args)
            .current_dir(&self.dir)
            .output()
            .expect("Unable to spawn command")
    }

    pub fn binary(&self) -> PathBuf {
        self.root.join("..").join("doctave")
    }

    pub fn assert_file_exists<P: AsRef<Path>>(&self, name: P) {
        assert!(
            self.dir.join(name.as_ref()).exists(),
            format!("File {} does not exist", name.as_ref().display())
        );
    }
}
