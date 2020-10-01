#[macro_use]
extern crate indoc;

#[allow(dead_code)]
mod support;

use std::path::Path;
use support::*;

integration_test!(build_smoke_test, |area| {
    area.create_config();
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
    area.assert_contains(&index, "<title>Test Project</title>");
    area.assert_contains(&index, "<li>Look</li>");
    area.assert_contains(&index, "<li>At</li>");
    area.assert_contains(&index, "<li>My</li>");
    area.assert_contains(&index, "<li>List</li>");
});

integration_test!(build_navigation, |area| {
    area.create_config();
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
    area.assert_contains(&index, "<a href=\"/howto_build\">How-To Build</a>");
    area.assert_contains(&index, "<a href=\"/runbooks\">Runbooks</a>");

    area.assert_exists(Path::new("site").join("howto_build.html"));
    area.assert_exists(Path::new("site").join("runbooks.html"));

    let howto = Path::new("site").join("howto_build.html");
    area.assert_contains(
        &howto,
        "<a class=\"active\" href=\"/howto_build\">How-To Build</a>",
    );
});

integration_test!(build_navigation_nested, |area| {
    area.create_config();
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
    area.assert_contains(&index, "<a href=\"/nested\">Nested</a>");
    area.assert_contains(&index, "<a href=\"/nested/howto_build\">How-To Build</a>");

    area.assert_exists(Path::new("site").join("nested").join("index.html"));
    area.assert_exists(Path::new("site").join("nested").join("howto_build.html"));
});

integration_test!(mermaid_js, |area| {
    area.create_config();
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

integration_test!(search_index, |area| {
    area.create_config();
    area.write_file(
        "README.md",
        indoc! {"
        # An Search!
        ```
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    area.assert_exists(Path::new("site").join("search_index.json"));
});

integration_test!(frontmatter, |area| {
    area.create_config();
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

    let index = area.path.join("site").join("index.html");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = std::fs::read_to_string(&index).unwrap();

    let start = index.find("<div class='content'>").unwrap();
    let end = index.find("<div class='sidebar-right'>").unwrap();

    // Check that there is no line between the beginning and end of the content
    assert!(!index[start..end].contains("<hr />"));
});

integration_test!(page_nav, |area| {
    area.create_config();
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

    area.assert_contains(&index, "<li class='page-nav-level-1'>");
    area.assert_contains(&index, "  <a href='#this-1'>This</a>");
    area.assert_contains(&index, "<li class='page-nav-level-1'>");
    area.assert_contains(&index, "  <a href='#is-2'>Is</a>");
    area.assert_contains(&index, "<li class='page-nav-level-1'>");
    area.assert_contains(&index, "  <a href='#the-3'>The</a>");
    area.assert_contains(&index, "<li class='page-nav-level-1'>");
    area.assert_contains(&index, "  <a href='#end-4'>End</a>");
});

integration_test!(missing_directory_index, |area| {
    area.create_config();
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
    area.assert_contains(
        &nested_index,
        "This page was generated automatically by Doctave",
    );
});

integration_test!(missing_directory_index_root, |area| {
    area.create_config();
    area.mkdir(Path::new("docs"));

    area.write_file(Path::new("README.md"), b"# Some content");
    area.write_file(
        Path::new("docs").join("not_the_index.md"),
        b"# Some other content",
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    // Assert we auto-generated an index page
    let index = Path::new("site").join("index.html");
    area.refute_contains(&index, "This page was generated automatically by Doctave");
    area.assert_contains(&index, "Some content");
});

integration_test!(code_syntax_highlight, |area| {
    area.create_config();
    area.write_file(
        Path::new("README.md"),
        indoc! {"
    # Some code

    ```ruby
    class Parser
        def initialize(input)
            @input = input
        end
    end

    ```

    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");
    area.assert_contains(&index, "<code class=\"language-ruby\">");
});

integration_test!(assets_folder, |area| {
    area.create_config();
    area.mkdir(Path::new("docs").join("_assets"));

    area.write_file(Path::new("docs").join("README.md"), b"# Hi");
    area.write_file(
        Path::new("docs").join("_assets").join("custom_style.css"),
        b"body { background-color: pink !important }",
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let css = Path::new("site").join("assets").join("custom_style.css");
    area.assert_contains(&css, "body { background-color: pink !important }");

    let index = Path::new("site").join("index.html");
    area.refute_contains(&index, "<a href=\"/_assets\">_assets</a>");
});

integration_test!(custom_colors, |area| {
    area.write_file(Path::new("doctave.yaml"), indoc! {"
    ---
    title: Custom colors
    colors:
      main: \"#5f658a\"
    "}.as_bytes());

    area.write_file(Path::new("README.md"), b"# Hi");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let css = Path::new("site").join("assets").join("doctave-style.css");
    // Should contain the RGB value for #5f658a
    area.assert_contains(&css, "color: rgb(95,101,138);");
});

integration_test!(custom_colors_invalid, |area| {
    area.write_file(Path::new("doctave.yaml"), indoc! {"
    ---
    title: Custom colors
    colors:
      main: not-a-color
    "}.as_bytes());

    area.write_file(Path::new("README.md"), b"# Hi");

    let result = area.cmd(&["build"]);
    assert_failed(&result);
    assert_output(&result, "Could not parse color code \"not-a-color\" from doctave.yaml");
});
