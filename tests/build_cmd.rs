#[macro_use]
extern crate indoc;

#[allow(dead_code)]
mod support;

use std::path::Path;
use support::*;

integration_test!(build_smoke_test, |area| {
    area.write_file("README.md", indoc! {"
        # Some content

        This is some text

        * Look
        * At
        * My
        * List
    "}.as_bytes());

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");

    area.assert_contains(&index, "<h1>Some content</h1>");
    area.assert_contains(&index, "<p>This is some text</p>");
    area.assert_contains(&index, "<li>Look</li>");
    area.assert_contains(&index, "<li>At</li>");
    area.assert_contains(&index, "<li>My</li>");
    area.assert_contains(&index, "<li>List</li>");
});

integration_test!(build_navigation, |area| {
    area.mkdir("docs");
    area.write_file("README.md", b"# Some content");
    area.write_file("docs/howto_build.md", indoc! {"
        ---
        title: How-To Build
        ---

        # How-To Build
    "}.as_bytes());
    area.write_file("docs/runbooks.md", indoc! {"
        ---
        title: Runbooks
        ---

        # Runbooks
    "}.as_bytes());

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");
    area.assert_contains(&index, "<a href=\"/howto_build.html\">How-To Build</a>");
    area.assert_contains(&index, "<a href=\"/runbooks.html\">Runbooks</a>");

    area.assert_exists(Path::new("site").join("howto_build.html"));
    area.assert_exists(Path::new("site").join("runbooks.html"));
});
