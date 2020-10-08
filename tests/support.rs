use std::env::current_exe;
use std::ffi::OsStr;
use std::fs::{self, create_dir_all, remove_dir_all, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

static TEST_WORKDIRS: &'static str = "_test_area";

#[macro_export]
/// A macro for creating an integration test inside an isolated environment.
///
/// The called provides the name of the test, and a closure describing the body
/// of the test. The closure takes one argument, which is the TestArea struct
/// describing the isolated environment.
///
macro_rules! integration_test {
    ($name:ident, $body:expr) => {
        #[test]
        fn $name() {
            test_dir(stringify!($name), $body);
        }
    };
}

pub fn test_dir<F, P: AsRef<Path>>(name: P, lambda: F)
where
    F: FnOnce(&TestArea),
{
    let dir = TestArea::create(name).expect("Could not create test dir");

    (lambda)(&dir);
}

pub struct TestArea {
    pub path: PathBuf,
    project_root: PathBuf,
}

impl TestArea {
    pub fn create<P: AsRef<Path>>(name: P) -> io::Result<Self> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join(TEST_WORKDIRS)
            .join(name);

        let project_root = current_exe()
            .unwrap()
            .parent()
            .expect("executable's directory")
            .to_path_buf();

        if path.exists() {
            remove_dir_all(&path)?;
        }
        create_dir_all(&path)?;

        Ok(TestArea { path, project_root })
    }

    /// Runs the given command, with the current directory set as the test area.
    pub fn cmd<I, S>(&self, args: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        Command::new(self.binary())
            .args(args)
            .args(&["--no-color"]) // Disable color in tests
            .current_dir(&self.path)
            .output()
            .expect("Unable to spawn command")
    }

    /// The location of the doctave executable
    pub fn binary(&self) -> PathBuf {
        self.project_root.join("..").join("doctave")
    }

    pub fn mkdir<P: AsRef<Path>>(&self, name: P) {
        create_dir_all(self.path.join(name)).expect("Could not create dir");
    }

    pub fn create_config(&self) {
        let mut file = File::create(self.path.join("doctave.yaml")).unwrap();
        file.write(b"---\ntitle: Test Project\n").unwrap();
    }

    pub fn write_file<P: AsRef<Path>>(&self, name: P, content: &[u8]) {
        let mut file = File::create(self.path.join(name)).unwrap();
        file.write(content).unwrap();
    }

    pub fn assert_exists<P: AsRef<Path>>(&self, name: P) {
        assert!(
            self.path.join(name.as_ref()).exists(),
            format!(
                "Could not find '{}'. Only found {:?}",
                name.as_ref().display(),
                std::fs::read_dir(&self.path.join(name.as_ref()).parent().unwrap())
                    .unwrap()
                    .map(|e| e.unwrap().path().to_path_buf())
                    .map(|p| p.strip_prefix(&self.path).unwrap().to_path_buf())
                    .collect::<Vec<_>>()
            )
        );
    }

    pub fn refute_exists<P: AsRef<Path>>(&self, name: P) {
        assert!(
            !self.path.join(name.as_ref()).exists(),
            format!(
                "Found '{}' even though expected not to",
                name.as_ref().display(),
            )
        );
    }

    pub fn assert_contains<P: AsRef<Path>>(&self, name: P, needle: &str) {
        self.assert_exists(&name);

        let haystack = fs::read_to_string(self.path.join(&name)).unwrap();

        assert!(
            haystack.contains(needle),
            format!(
                "Could not find \"{}\" inside file \"{}\".\nFound:\n---\n{}\n---\n",
                needle,
                name.as_ref().display(),
                haystack
            )
        );
    }

    pub fn refute_contains<P: AsRef<Path>>(&self, name: P, needle: &str) {
        self.assert_exists(&name);

        let haystack = fs::read_to_string(self.path.join(&name)).unwrap();

        assert!(
            !haystack.contains(needle),
            format!(
                "Found \"{}\" inside file \"{}\", when it was not expected",
                needle,
                name.as_ref().display(),
            )
        );
    }
}

pub fn assert_success(result: &std::process::Output) {
    assert!(
        result.status.success(),
        format!(
            "Command was not successful! \nSTDOUT:\n---\n{}\n--- \n\nSTDERR:\n---\n{}\n---",
            std::str::from_utf8(&result.stdout).unwrap(),
            std::str::from_utf8(&result.stderr).unwrap()
        )
    );
}

pub fn assert_failed(result: &std::process::Output) {
    assert!(
        !result.status.success(),
        format!(
            "Command was unexpectedly successful! \nSTDOUT:\n---\n{}\n--- \n\nSTDERR:\n---\n{}\n---",
            std::str::from_utf8(&result.stdout).unwrap(),
            std::str::from_utf8(&result.stderr).unwrap()
            )
        );
}

pub fn assert_output(result: &std::process::Output, needle: &str) {
    let stdout = std::str::from_utf8(&result.stdout).unwrap();
    let stderr = std::str::from_utf8(&result.stderr).unwrap();

    assert!(
        stdout.contains(needle) || stderr.contains(needle),
        format!(
            "Could not find \"{}\" in STDOUT or STDERR:\n\n------ STDOUT ------\n{}\n \
            ------ STDERR ------\n{}\n",
            needle, stdout, stderr
        )
    )
}

pub fn refute_output(result: &std::process::Output, needle: &str) {
    let stdout = std::str::from_utf8(&result.stdout).unwrap();
    let stderr = std::str::from_utf8(&result.stderr).unwrap();

    assert!(
        !stdout.contains(needle) && !stderr.contains(needle),
        format!(
            "Found {} in the command output, even though it shoudn't be there: \
            \n\n------ STDOUT ------\n{}\n------ STDERR --------\n{}\n",
            needle, stdout, stderr
        )
    );
}
