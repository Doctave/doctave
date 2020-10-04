---
title: Configuration
---

Configuration
=============

This document goes through all the different configuration options available.

## Doctave.yaml

First, let's look at the options available in the `doctave.yaml` file.

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

### colors.main

This sets the main color for your site. You can read more about this in
[customization](/features/customization). Currently this is the only color you can customize.

This is an optional setting.

This setting _must be a valid hex value_.

Example:

```yaml
---
colors:
  main: #FF78E4

```

### logo

The name of the file to serve as your logo. You can read more about this in
[customization](/features/customization).

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

## Serve command

Currently the `serve` command takes only one optional argument.

### --port, -p

Sets the port the development server will listen on when running the `serve` command.

This is an optional argument.

Example:

```
$ doctave serve --port 5432
```
