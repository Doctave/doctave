mod support;

use support::*;

#[test]
fn init_smoke_test() {
    test_dir("init_smoke_test", |dir| {
        let result = dir.cmd(&["init"]);
        assert!(result.status.success());

        println!("{}", std::str::from_utf8(&result.stdout).unwrap());

        dir.assert_file_exists("README.md");
    });
}
