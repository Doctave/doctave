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

integration_test!(build_navigation_nested, |area| {
    area.mkdir("docs");
    area.mkdir("docs/nested");
    area.write_file("README.md", b"# Some content");
    area.write_file("docs/runbooks.md", indoc! {"
        ---
        title: Runbooks
        ---

        # Runbooks
    "}.as_bytes());
    area.write_file("docs/nested/README.md", indoc! {"
        ---
        title: Nested
        ---

        # How-To Build
    "}.as_bytes());
    area.write_file("docs/nested/howto_build.md", indoc! {"
        ---
        title: How-To Build
        ---

        # How-To Build
    "}.as_bytes());

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");
    area.assert_contains(&index, "<a href=\"/nested/index.html\">Nested</a>");
    area.assert_contains(&index, "<a href=\"/nested/howto_build.html\">How-To Build</a>");

    area.assert_exists(Path::new("site").join("nested").join("index.html"));
    area.assert_exists(Path::new("site").join("nested").join("howto_build.html"));
});

integration_test!(mermaid_js, |area| {
    area.write_file("README.md", indoc! {"
        # Mermaid 

        ```mermaid
        graph TD
          A[Christmas] -->|Get money| B(Go shopping)
          B --> C{Let me think}
          C -->|One| D[Laptop]
          C -->|Two| E[iPhone]
          C -->|Three| F[fa:fa-car Car]
        ```
    "}.as_bytes());

    let index = Path::new("site").join("index.html");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    area.assert_contains(&index, "<h1>Mermaid</h1>");
    area.assert_contains(&index, "<div class=\"mermaid\">");
    area.assert_contains(&index, "Car]\n</div>");
});

integration_test!(regular_code, |area| {
    area.write_file("README.md", indoc! {"
        # Code Block 

        ```ruby
        1 + 1
        ```
    "}.as_bytes());

    let index = Path::new("site").join("index.html");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    area.assert_contains(&index, "<h1>Code Block</h1>");
    area.assert_contains(&index, "<code class=\"language-ruby\">");
});
