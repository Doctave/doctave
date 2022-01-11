#[macro_use]
extern crate indoc;

#[allow(dead_code)]
mod support;

use std::path::Path;
use support::*;

integration_test!(build_smoke_test, |area| {
    area.create_config();
    area.mkdir("docs");
    area.write_file(
        Path::new("docs").join("README.md"),
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
    area.write_file(Path::new("docs").join("README.md"), b"# Some content");
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
    area.write_file(Path::new("docs").join("README.md"), b"# Some content");
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
    area.mkdir("docs");
    area.create_config();
    area.write_file(
        Path::new("docs").join("README.md"),
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
    area.mkdir("docs");
    area.create_config();
    area.write_file(
        Path::new("docs").join("README.md"),
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
    area.mkdir("docs");
    area.create_config();
    area.write_file(
        Path::new("docs").join("README.md"),
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

    let start = index.find("<div class='doctave-content'>").unwrap();
    let end = index.find("<div class='sidebar-right'>").unwrap();

    // Check that there is no line between the beginning and end of the content
    assert!(!index[start..end].contains("<hr />"));
});

integration_test!(page_nav, |area| {
    area.mkdir("docs");
    area.create_config();
    area.write_file(
        Path::new("docs").join("README.md"),
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
    area.assert_contains(&index, "  <a href='#this'>This</a>");
    area.assert_contains(&index, "<li class='page-nav-level-1'>");
    area.assert_contains(&index, "  <a href='#is'>Is</a>");
    area.assert_contains(&index, "<li class='page-nav-level-1'>");
    area.assert_contains(&index, "  <a href='#the'>The</a>");
    area.assert_contains(&index, "<li class='page-nav-level-1'>");
    area.assert_contains(&index, "  <a href='#end'>End</a>");
});

integration_test!(no_table_contents, |area| {
    area.mkdir(Path::new("docs"));
    area.write_file(
        Path::new("doctave.yaml"),
        indoc! {"
    ---
    title: Custom colors
    table_contents: false
    "}
        .as_bytes(),
    );

    area.write_file(
        Path::new("docs").join("README.md"),
        b"# Hi\n## Foo\n### Bar",
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");
    area.refute_contains(&index, "On this page");
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

integration_test!(code_syntax_highlight, |area| {
    area.create_config();
    area.mkdir(Path::new("docs"));
    area.write_file(
        Path::new("docs").join("README.md"),
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

integration_test!(include_folder, |area| {
    area.create_config();

    area.mkdir(Path::new("docs"));
    area.write_file(Path::new("docs").join("README.md"), b"# Hi");

    area.mkdir(Path::new("docs").join("_include"));
    area.write_file(
        Path::new("docs").join("_include").join("an_file"),
        b"an content",
    );

    area.mkdir(Path::new("docs").join("_include").join("assets"));
    area.write_file(
        Path::new("docs")
            .join("_include")
            .join("assets")
            .join("an_nested_file"),
        b"an nested content",
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let css = area.path.join("site").join("an_file");
    area.assert_contains(&css, "an content");

    let css = area.path.join("site").join("assets").join("an_nested_file");
    area.assert_contains(&css, "an nested content");

    let index = area.path.join("site").join("index.html");
    area.refute_contains(&index, "<a href=\"/_assets\">_assets</a>");
});

integration_test!(custom_colors, |area| {
    area.mkdir(Path::new("docs"));
    area.write_file(
        Path::new("doctave.yaml"),
        indoc! {"
    ---
    title: Custom colors
    colors:
      main: \"#5f658a\"
    "}
        .as_bytes(),
    );

    area.write_file(Path::new("docs").join("README.md"), b"# Hi");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let css = Path::new("site").join("assets").join("doctave-style.css");
    // Should contain the RGB value for #5f658a
    area.assert_contains(&css, "color: rgb(95,101,138);");
});

integration_test!(custom_colors_invalid, |area| {
    area.mkdir(Path::new("docs"));
    area.write_file(
        Path::new("doctave.yaml"),
        indoc! {"
    ---
    title: Custom colors
    colors:
      main: not-a-color
    "}
        .as_bytes(),
    );

    area.write_file(Path::new("docs").join("README.md"), b"# Hi");

    let result = area.cmd(&["build"]);
    assert_failed(&result);
    assert_output(
        &result,
        "Invalid HEX color provided for colors.main in doctave.yaml.",
    );
    assert_output(&result, "Found 'not-a-color'");
});

integration_test!(release_mode, |area| {
    area.create_config();
    area.mkdir(Path::new("docs"));
    area.write_file(Path::new("docs").join("README.md"), b"# Hi");

    let result = area.cmd(&["build", "--release"]);
    assert_success(&result);

    let index = area.path.join("site").join("index.html");
    area.refute_contains(&index, "livereload");

    let livereload_js = area.path.join("site").join("assets").join("livereload.js");
    assert!(!livereload_js.exists());
});

integration_test!(custom_logo, |area| {
    area.mkdir(Path::new("docs").join("_include").join("assets"));
    area.write_file(Path::new("docs").join("README.md"), b"# Hi");

    // Create a fake logo
    area.write_file(
        Path::new("docs")
            .join("_include")
            .join("assets")
            .join("fake-logo.png"),
        b"",
    );

    // Include the logo in the config
    area.write_file(
        Path::new("doctave.yaml"),
        indoc! {"
    ---
    title: Custom colors
    logo: assets/fake-logo.png
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");

    area.assert_contains(&index, "/assets/fake-logo.png");
});

integration_test!(include_header, |area| {
    area.create_config();
    area.mkdir(Path::new("docs").join("_include"));
    area.write_file(Path::new("docs").join("README.md"), b"# Hi");
    area.write_file(
        Path::new("docs").join("_include").join("_head.html"),
        b"<script>console.log(1 + 1)</script>",
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");

    area.assert_contains(&index, "<script>console.log(1 + 1)</script>");

    let head = Path::new("site").join("_head.html");
    area.refute_exists(&head);
});

integration_test!(cache_buster, |area| {
    area.create_config();
    area.mkdir("docs");
    area.write_file(Path::new("docs").join("README.md"), b"# Hi");

    let result = area.cmd(&["build"]);
    assert_success(&result);

    let index = Path::new("site").join("index.html");

    // No access to the actual timestamp, but we should be fine until unix timestamps
    // roll over to start with the number 2.
    //
    // Famous last words ofc...
    area.assert_contains(&index, "doctave-style.css?v=1");
});

integration_test!(base_path, |area| {
    area.create_config();
    area.mkdir("docs");
    area.write_file(Path::new("docs").join("README.md"), b"[link](/foo)");
    area.write_file(Path::new("docs").join("foo.md"), b"[link](/)");
    area.write_file(
        Path::new("doctave.yaml"),
        indoc! {"
    ---
    title: Base Path
    base_path: /docs
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);
    println!("{}", std::str::from_utf8(&result.stdout).unwrap());

    let index = area.path.join("site/index.html");

    area.refute_contains(&index, "<a href=\"/\">");
    area.refute_contains(&index, "<a href='/'>");

    area.refute_contains(&index, "<a href=\"/foo\">");
    area.assert_contains(&index, "<a href=\"/docs/foo\">");
});

integration_test!(base_path_with_custom_navigation, |area| {
    area.create_config();
    area.mkdir("docs");
    area.write_file(Path::new("docs").join("README.md"), b"[link](/other)");
    area.write_file(Path::new("docs").join("other.md"), b"[link](/)");
    area.write_file(
        Path::new("doctave.yaml"),
        indoc! {"
    ---
    title: Base Path
    base_path: /docs
    navigation:
        - path: docs/other.md
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);
    println!("{}", std::str::from_utf8(&result.stdout).unwrap());

    let index = area.path.join("site/index.html");

    area.refute_contains(&index, "<a href=\"/\">");
    area.refute_contains(&index, "<a href='/'>");

    area.refute_contains(&index, "<a href=\"/other\">");
    area.assert_contains(&index, "<a href=\"/docs/other\">");
});

integration_test!(base_path_with_logo, |area| {
    area.create_config();
    area.mkdir(Path::new("docs").join("_include").join("assets"));
    area.write_file(Path::new("docs").join("README.md"), b"[link](/foo)");
    area.write_file(Path::new("docs").join("foo.md"), b"# Foo");
    // Create a fake logo
    area.write_file(
        Path::new("docs")
            .join("_include")
            .join("assets")
            .join("fake-logo.png"),
        b"",
    );
    area.write_file(
        Path::new("doctave.yaml"),
        indoc! {"
    ---
    title: Base Path
    base_path: /docs
    logo: assets/fake-logo.png
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);
    println!("{}", std::str::from_utf8(&result.stdout).unwrap());

    let index = area.path.join("site").join("index.html");
    area.assert_contains(&index, "/assets/fake-logo.png");

    area.refute_contains(&index, "<a href=\"/\">");
    area.refute_contains(&index, "<a href='/'>");
});

// See (Issue 18)[https://github.com/Doctave/doctave/issues/18]
integration_test!(issue_18, |area| {
    area.write_file(
        Path::new("doctave.yaml"),
        indoc! {"
    ---
    title: Test project
    navigation:
        - path: docs/README.md
        - path: docs/another_file.md
    "}
        .as_bytes(),
    );
    area.mkdir("docs");

    area.write_file(
        Path::new("docs").join("README.md"),
        indoc! {"# A test file"}.as_bytes(),
    );
    area.write_file(
        Path::new("docs").join("another_file.md"),
        indoc! {"# A second test file"}.as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);
});

integration_test!(broken_link_detection, |area| {
    area.create_config();
    area.mkdir("docs");
    area.write_file(
        Path::new("docs").join("README.md"),
        indoc! {"

        [Road to nowhere](/nope)
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_failed(&result);

    let stdout = std::str::from_utf8(&result.stdout).unwrap();

    println!("{}", stdout);

    assert!(stdout.contains("Detected broken internal links"));
    assert!(stdout.contains("/nope"));
    assert!(stdout.contains("Road to nowhere"));
});

integration_test!(broken_link_detection_can_be_skipped_with_flag, |area| {
    area.create_config();
    area.mkdir("docs");
    area.write_file(
        Path::new("docs").join("README.md"),
        indoc! {"

        [Road to nowhere](/nope)
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build", "--allow-failed-checks"]);
    assert_success(&result);

    let stdout = std::str::from_utf8(&result.stdout).unwrap();

    println!("{}", stdout);

    assert!(stdout.contains("Detected broken internal links"));
    assert!(stdout.contains("/nope"));
    assert!(stdout.contains("Road to nowhere"));
});

integration_test!(includes_katex_bundles, |area| {
    area.create_config();
    area.mkdir("docs");
    area.write_file(
        Path::new("docs").join("README.md"),
        indoc! {"
        # New phone, who dis?
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    area.assert_exists(area.path.join("site").join("assets").join("katex-fonts"));
    area.assert_exists(area.path.join("site").join("assets").join("katex.js"));
    area.assert_exists(area.path.join("site").join("assets").join("katex.css"));
});

integration_test!(includes_prism_grammars, |area| {
    area.create_config();
    area.mkdir("docs");
    area.write_file(
        Path::new("docs").join("README.md"),
        indoc! {"
        # New phone, who dis?
    "}
        .as_bytes(),
    );

    let result = area.cmd(&["build"]);
    assert_success(&result);

    area.assert_exists(area.path.join("site").join("assets").join("prism-grammars"));
    area.assert_exists(area.path.join("site").join("assets").join("prism.js"));
});
