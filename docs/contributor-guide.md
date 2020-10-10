---
title: Contributor guide
---

# Contributor guide

Firstly, **thank you for considering contributing to Doctave**. Open source software would not
survive without people like yourself, who take time out of their busy schedules to help out. **We
appreciate you.**

This document will help you get started working with Doctave. It will walk you through how to build,
run tests, and things to keep in mind when contributing.

First, make sure you've cloned the repo,

```
git clone git@github.com:Doctave/doctave.git
```

and installed the [Rust toolchain](https://www.rust-lang.org/learn/get-started).

## Building


Doctave is a fairly standard Rust project. It uses Cargo, doesn't have any non-rust dependencies, so
assuming you have Rust installed, you should be able to run `cargo build` and get a working build.
If not, please open an issue on Github with the error you see.

```
$ cargo build

...building ensues...
```

You can now verify your installation works with:

```
$ cargo run -- --version
```

## Running tests

Again, Doctave uses Cargo. You can run all tests with

```
$ cargo test
```

This will run unit tests and integration tests.


### Test structure

Doctave has two types of tests: unit tests and integration tests.

You will find unit tests at the bottom of source files under `src`. As a general rule, they should
not have any external dependencies - especially on the file system.

Integrations tests live under the `tests` directory. These tests shell out and execute the doctave
binary directly inside an isolated environment. Each test will get its own directory under the
`_test_area` directory when executed.

### Adding integration tests

If you want to add integration tests, you should use the `integration_test` macro to define your
tests. It takes two arguments: the name of the test, and a lambda to run the test code. The argument
passed into the lambda is a `TestArea` struct describing the isolated test area. When creating
files, or running commands or assertions, you should always run them via this struct. The definition
of the struct is under `docs/support.rs`.

Below is a simple example of a smoke test that checks when a site is built, it will contain the
expected content in the index file.

```rust
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
```

## Cross-platform compatibility

Doctave runs on Mac, Linux, and Windows, which means you need to be careful about not relying on
platform-specific features. Most commonly for Doctave, this means you have to be careful when
building filesystem paths.

Luckily, Rust has a good solution for this: the `Path` and `PathBuf` structs. If you are familiar
with Rust, think of these as `str` and `String` equivalents for handling paths. `Path` is always
accessed through a reference, while `PathBuf` is an owned type.

## CI

Doctave uses [GitHub actions](https://github.com/Doctave/doctave/actions) to run tests
automatically for each commit and PR. The test suite is executed on Mac, Linux, and Windows. When
opening a PR, make sure you pass all checks.
