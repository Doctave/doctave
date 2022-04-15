---
title: Checks
---

Checks
======

Doctave will over time add various checks that can be run as part of your build. When you run `doctave build`, you will
see any failed checks in the terminal output. To not error out on these checks, use the `--allow-failed-checks` flag.

Currently the only supported check is broken links checking.

## Broken Links

Broken links are links that point to pages that do not exist. Over time as you update your documentation, your links may
become out of date as content is moved around. This check verifies that any internal links that you have in your
documentation refer to pages that exist.

You don't have to do anything to enable this feature - it is on by default. While in `serve` mode, you will see broken
links as warnings in the terminal output. When running a `build`, any broken links will fail the build by default.

Below is some example output for the `serve` command:

```plain
$ doctave serve

...

WARNING
Detected broken internal links.
The following links point to pages that do not exist:

	features/markdown.md : [I don't exist](/nothing/here)

```

And the `build` command:

```plain
$ doctave build

...

ERROR: Detected broken internal links.
The following links point to pages that do not exist:

	features/markdown.md : [I don't exist](/nothing/here)

```

### Limitations

* Only interal links within a Doctave project are checked
* Anchor tags are not verified
