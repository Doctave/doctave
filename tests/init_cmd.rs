#[allow(dead_code)]
mod support;

use std::path::Path;
use support::*;

integration_test!(init_smoke_test, |area| {
    let result = area.cmd(&["init"]);

    assert_success(&result);

    area.assert_exists("README.md");
    area.assert_exists("docs");
});

integration_test!(does_not_overwite_readme, |area| {
    area.write_file("README.md", b"Some content");

    let result = area.cmd(&["init"]);
    assert_success(&result);

    area.assert_contains("README.md", "Some content");
    area.refute_contains("README.md", "Hello, world:");
});

integration_test!(does_not_overwite_existing_docs, |area| {
    area.mkdir("docs");
    area.write_file(Path::new("docs").join("some_file.md"), b"Some content");

    let result = area.cmd(&["init"]);
    assert_success(&result);

    area.assert_contains(Path::new("docs").join("some_file.md"), "Some content");
});
