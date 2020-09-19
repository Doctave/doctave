#[macro_use]
extern crate indoc;

#[allow(dead_code)]
mod support;

use std::path::Path;
use support::*;

integration_test!(build_smoke_test, |area| {
    area.write_file(
        "README.md",
        indoc! {"
        # Some content

        This is some text

        * Look
        * At
        * My
        * List
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");

    area.assert_contains(&index, ">Some content</h1>");
    area.assert_contains(&index, "<p>This is some text</p>");
    area.assert_contains(&index, "<li>Look</li>");
    area.assert_contains(&index, "<li>At</li>");
    area.assert_contains(&index, "<li>My</li>");
    area.assert_contains(&index, "<li>List</li>");
});

integration_test!(build_navigation, |area| {
    area.mkdir("docs");
    area.write_file("README.md", b"# Some content");
    area.write_file(
        Path::new("docs").join("howto_build.md"),
        indoc! {"
        ---
        title: How-To Build
        ---

        # How-To Build
    "}
        .as_bytes(),
    );
    area.write_file(
        "docs/runbooks.md",
        indoc! {"
        ---
        title: Runbooks
        ---

        # Runbooks
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");
    area.assert_contains(&index, "<a href=\"/howto_build.html\">How-To Build</a>");
    area.assert_contains(&index, "<a href=\"/runbooks.html\">Runbooks</a>");

    area.assert_exists(Path::new("site").join("howto_build.html"));
    area.assert_exists(Path::new("site").join("runbooks.html"));

    let howto = Path::new("site").join("howto_build.html");
    area.assert_contains(
        &howto,
        "<a class=\"active\" href=\"/howto_build.html\">How-To Build</a>",
    );
});

integration_test!(build_navigation_nested, |area| {
    area.mkdir("docs");
    area.mkdir(Path::new("docs").join("nested"));
    area.write_file("README.md", b"# Some content");
    area.write_file(
        Path::new("docs").join("runbooks.md"),
        indoc! {"
        ---
        title: Runbooks
        ---

        # Runbooks
    "}
        .as_bytes(),
    );
    area.write_file(
        "docs/nested/README.md",
        indoc! {"
        ---
        title: Nested
        ---

        # How-To Build
    "}
        .as_bytes(),
    );
    area.write_file(
        "docs/nested/howto_build.md",
        indoc! {"
        ---
        title: How-To Build
        ---

        # How-To Build
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");
    area.assert_contains(&index, "<a href=\"/nested/index.html\">Nested</a>");
    area.assert_contains(
        &index,
        "<a href=\"/nested/howto_build.html\">How-To Build</a>",
    );

    area.assert_exists(Path::new("site").join("nested").join("index.html"));
    area.assert_exists(Path::new("site").join("nested").join("howto_build.html"));
});

integration_test!(mermaid_js, |area| {
    area.write_file(
        "README.md",
        indoc! {"
        # Mermaid 

        ```mermaid
        graph TD
          A[Christmas] -->|Get money| B(Go shopping)
          B --> C{Let me think}
          C -->|One| D[Laptop]
          C -->|Two| E[iPhone]
          C -->|Three| F[fa:fa-car Car]
        ```
    "}
        .as_bytes(),
    );

    let index = Path::new("site").join("index.html");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    area.assert_contains(&index, ">Mermaid</h1>");
    area.assert_contains(&index, "<div class=\"mermaid\">");
    area.assert_contains(&index, "Car]\n</div>");
});

integration_test!(regular_code, |area| {
    area.write_file(
        "README.md",
        indoc! {"
        # Code Block 

        ```ruby
        1 + 1
        ```
    "}
        .as_bytes(),
    );

    let index = Path::new("site").join("index.html");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    area.assert_contains(&index, ">Code Block</h1>");
    area.assert_contains(&index, "<code class=\"language-ruby\">");
});

integration_test!(search_index, |area| {
    area.write_file(
        "README.md",
        indoc! {"
        # Code Block 

        ```ruby
        1 + 1
        ```
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let search_index = Path::new("site").join("search_index.json");

    area.assert_contains(&search_index, "Code Block");
    area.assert_contains(&search_index, "1 + 1");
});

integration_test!(frontmatter, |area| {
    area.write_file(
        "README.md",
        indoc! {"
        ---
        title: \"The start\"
        ---

        # This is the end
        ```
    "}
        .as_bytes(),
    );

    let index = Path::new("site").join("index.html");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    area.assert_contains(&index, ">This is the end</h1>");
    area.refute_contains(&index, "<hr />");
});

integration_test!(page_nav, |area| {
    area.write_file(
        "README.md",
        indoc! {"
        # This

        # Is

        # The

        # End
        ```
    "}
        .as_bytes(),
    );

    let index = Path::new("site").join("index.html");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    area.assert_contains(
        &index,
        "<a class='page-nav-level-1' href='#this-1'>This</a>",
    );
    area.assert_contains(&index, "<a class='page-nav-level-1' href='#is-2'>Is</a>");
    area.assert_contains(&index, "<a class='page-nav-level-1' href='#the-3'>The</a>");
    area.assert_contains(&index, "<a class='page-nav-level-1' href='#end-4'>End</a>");
});

integration_test!(missing_directory_index, |area| {
    area.mkdir(Path::new("docs").join("nested"));

    area.write_file(Path::new("docs").join("README.md"), b"# Some content");
    area.write_file(
        Path::new("docs").join("nested").join("not_the_index.md"),
        b"# Some other content",
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    // Assert we auto-generated an index page
    let nested_index = Path::new("site").join("nested").join("index.html");
    area.assert_contains(&nested_index, "This page was generated automatically by Doctave");
});

integration_test!(missing_directory_index_root, |area| {
    area.mkdir(Path::new("docs"));

    area.write_file(Path::new("README.md"), b"# Some content");
    area.write_file(
        Path::new("docs").join("not_the_index.md"),
        b"# Some other content",
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    // Assert we auto-generated an index page
    let nested_index = Path::new("site").join("index.html");
    area.refute_contains(&nested_index, "This page was generated automatically by Doctave");
    area.assert_contains(&nested_index, "Some content");
});
