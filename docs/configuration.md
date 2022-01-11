---
title: Configuration
---

Configuration
=============

This document goes through all the different configuration options available.

## Doctave.yaml

First, let's look at the options available in the `doctave.yaml` file.

{% info :warning: Remember to restart %}
Any changes you make to this file will only come into effect when you restart the preview
server, or rebuild your site.
{% end %}

### title

This sets the name of your project. It serves two purposes:

* It will displayed as the title at the top right of the page
* The HTML page title will be set to this when you're on the root page

This is a **required** setting.

Example:

```yaml
---
title: Authentication service

```

### port

Sets the port the development server will listen on when running the `serve` command.

This is an optional setting.

This setting _must be a positive integer_.

Example:
```yaml
---
port: 5432
```

### base_path

Tells Doctave to generate all URLs based on a subpath. Use this if you are deploying your site under
a subdirectory, such as `https://example.com/docs`.

You won't have to change any URLs inside your docs when this value is changed. You can stil
construct paths to other pages and assets as if the site was served from the root of the URL. E.g.
if you have a page `docs/deployment/workflow.md`, you can link to it with `/deployment/workflow`,
without worrying about the base_path.

This is an optional setting.

This setting _must be an absolute path_.

Example:
```yaml
---
base_path: /docs
```

### docs_dir

Tells Doctave to use the specified path as the base path for generating documentation.

This option should be used if you wish to store your documentation in a directory that is not `docs/`.

This is an optional setting.

Example:
```yaml
---
docs_dir: custom_documentation_directory/
---
```

### colors.main

This sets the main color for your site. You can read more about this in the
[look-and-feel section](/features/look-and-feel). Currently this is the only color you can customize.

This is an optional setting.

This setting _must be a valid hex value_.

Example:

```yaml
---
colors:
  main: #FF78E4

```

### logo

The name of the file to serve as your logo. You can read more about this in the
[look-and-feel section](/features/look-and-feel).

This is an optional setting.

```yaml
---
logo: logo.png
```

### navigation

Customizes your site navigation on the left side of the page.

You can read more about this under [custom navigation](/features/custom-navigation.md).

This is an optional setting.

```yaml
navigation:
  - path: docs/installing.md
  - path: docs/tutorial.md
  - path: docs/features
    children: "*"
```


### table of contents

Tells Doctave to show or not the table of contents in the right side of the main content.

This is an optional setting.

```yaml
table_contents: true
```

## All commands

All commands support the following option.

### --no-color

Disable terminal colors.

This is an optional argument.

Example:

```
$ doctave serve --no-color
```

## Serve command

Currently the `serve` command takes only one optional argument.

### --port, -p

Sets the port the development server will listen on when running the `serve` command.

This is an optional argument.

Example:

```
$ doctave serve --port 5432
```

## Build command

Currently the `build` command takes only one optional argument.

### --release

This flag will build the site without development dependencies. Currently this means stripping out
livereload.js from the bundle, but can be extended in the future to include other actions.

This is an optional argument.

Example:

```
$ doctave build --release
```
