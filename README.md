Doctave CLI
===========

The [Doctave](https://www.doctave.com) CLI is an opinionated documentation site generator that
converts your Markdown files into a beautiful developer hub.

The Doctave CLI is not a generic static site generator - it is only meant for generating
documentation sites from Markdown. This allows the tool to be much simpler than other solutions,
with fewer configuration steps.

This open source CLI is built and maintained by Doctave. While it is meant originally to be used for
projects that are hosted on Doctave, you are free to deploy the generated sites as you please.

You can read more about hosting your docs on Doctave [here](https://www.doctave.com).

## Features
- [x] Zero-configuration
- [x] Supports GitHub flavored markdown
- [x] Full-text search built-in
- [x] Fast build, built in Rust
- [x] Local live-reloading preview

## Usage

The CLI comes with a number of sub commands:

### Starting a new project 

To initialize a Doctave site in your repository, run the following command:

```bash
$ doctave init
```

This will generate the following default directory structure, along with some boilerplate content
to get you started.

```bash
$ tree
.
├── README.md
└── docs
    ├── environment_setup.rs
    ├── architecture_overview.rs
    └── decision_records.rs
```

### Live preview

To view your documentation locally, you can start the development server (port 4001 by default):

```bash
$ doctave serve
```

You can now see your site at [localhost:4001](), and any updates to your docs will automatically reload the page.

### Building your site

Finally, to build a production version of your documentation, run the following:

```bash
$ doctave build
```
