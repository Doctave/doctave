![CI](https://github.com/Doctave/doctave-cli/workflows/CI/badge.svg)

![Doctave logo](./src/assets/gh-readme-logo.png)

Doctave CLI
===========

* [Docs](https://cli.doctave.com) (build with Doctave)
* [Tutorial](https://cli.doctave.com/tutorial)

The DoctaveCLI is an opinionated documentation site generator that converts your Markdown files into
a beautiful documentation site with minimal effort.

The Doctave CLI is not a generic static site generator - it is only meant for generating
documentation sites from Markdown. This allows the tool to be much simpler than other solutions,
with fewer configuration steps.

This open source CLI is built and maintained by [Doctave](https://www.doctave.com).

## Features

Doctave comes with a number of documentation-specific features out of the box. No plugins needed.

- [Mermaid.js](https://mermaid-js.github.io/) diagrams
- Full-text search
- Local live-reloading server
- Responsive design
- Dark mode
- GitHub flavored markdown
- Minimal configuration
- Fast build, built in Rust

## Screenshots

You can customize the color scheme and logo to match your own style. Below are two examples: one
with Doctave's own color scheme, and another customized color scheme.

Light                                             | Dark                                                    |
--------------------------------------------------|---------------------------------------------------------|
![Exmple 1](./docs/_include/assets/example-1.png) | ![Example 2](./docs/_include/assets/example-1-dark.png) |
![Exmple 2](./docs/_include/assets/example-2.png) | ![Example 2](./docs/_include/assets/example-2-dark.png) |

## Installation

> TODO

## Getting started

Once you have Doctave installed, you can run the `init` command to create an initial docs site:

```
$ doctave init
```

Then, run the `serve` command to preview your site locally.

```
Doctave CLI | Serve
🚀 Starting development server...

Server running on http://0.0.0.0:4001/

```
