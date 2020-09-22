#[allow(dead_code)]
mod support;

#[macro_use]
extern crate indoc;

use std::path::Path;
use support::*;

integration_test!(init_smoke_test, |area| {
    let result = area.cmd(&["init"]);

    assert_success(&result);

    assert_output(&result, "Created doctave.yaml...");
    assert_output(&result, "Created README.md...");
    assert_output(&result, "Created ./docs folder...");

    assert_output(&result, "Done! Run doctave serve to view your docs site locally.");

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

integration_test!(creates_doctave_yaml, |area| {
    let result = area.cmd(&["init"]);
    assert_success(&result);

    area.assert_contains(Path::new("doctave.yaml"), indoc! {"
    ---
    title: \"My Project\"
    "});
});

integration_test!(bails_if_doctave_yaml_already_exists, |area| {
    area.write_file(Path::new("doctave.yaml"), b"---\ntitle: I exist\n");

    let result = area.cmd(&["init"]);
    assert_failed(&result);

    assert_output(&result, "Aborting. Found an existing doctave.yaml.");
    assert_output(&result, "Have you already run `doctave init`?");
});
