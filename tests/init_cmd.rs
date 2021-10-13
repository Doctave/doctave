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
    assert_output(&result, "Created docs folder...");
    assert_output(&result, &format!("Created docs{}README.md...", std::path::MAIN_SEPARATOR));

    assert_output(
        &result,
        "Done! Run doctave serve to view your docs site locally.",
    );

    area.assert_exists(Path::new("docs").join("README.md"));
    area.assert_exists(Path::new("docs").join("examples.md"));
    area.assert_exists(Path::new("doctave.yaml"));
});

integration_test!(does_not_overwite_existing_docs, |area| {
    area.mkdir("docs");
    area.write_file(Path::new("docs").join("some_file.md"), b"Some content");

    let result = area.cmd(&["init"]);
    assert_success(&result);

    area.assert_contains(Path::new("docs").join("some_file.md"), "Some content");
});

integration_test!(parses_frontmatter_correctly_after_generating_pages_bug_8, |area| {
    // https://github.com/Doctave/doctave/issues/8
    //
    // When we generate pages on `init` on Windows we end up automatically using
    // Windows line endings. This seems to then confuse the frontmatter parser.
    //
    // Other tests did not pick this up since the tests were always adding unix
    // line endings. This tests runs the `init` task natively on the target platform
    // and then builds the site. We should see that the frontmatter is parsed
    // correctly.

    let result = area.cmd(&["init"]);
    assert_success(&result);

    area.assert_contains(Path::new("docs").join("examples.md"), "title: Examples");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    area.refute_contains(Path::new("site").join("examples.html"), "title: Examples");
});

integration_test!(creates_doctave_yaml, |area| {
    let result = area.cmd(&["init"]);
    assert_success(&result);

    area.assert_contains(
        Path::new("doctave.yaml"),
        indoc! {"
    ---
    title: \"My Project\"
    "},
    );
});

integration_test!(bails_if_doctave_yaml_already_exists, |area| {
    area.write_file(Path::new("doctave.yaml"), b"---\ntitle: I exist\n");

    let result = area.cmd(&["init"]);
    assert_failed(&result);

    assert_output(&result, "Aborting. Found an existing doctave.yaml.");
    assert_output(&result, "Have you already run doctave init?");
});

integration_test!(skips_generating_docs_if_docs_folder_exists, |area| {
    area.mkdir(Path::new("docs"));

    let result = area.cmd(&["init"]);
    assert_success(&result);

    assert_output(&result, "Skipping docs directory - found existing docs");

    area.refute_exists(Path::new("docs").join("README.md"));
    area.refute_exists(Path::new("docs").join("examples.md"));
    area.assert_exists(Path::new("doctave.yaml"));
});
