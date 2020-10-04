---
title: Custom navigation
---

Custom navigation
=================

By default, Doctave will include all your pages in the left-side navigation and sort them in
alphanumerical order. But sometimes you will want to customize either the order or the content of
the navigation. This is why you can set the contents of the navigation in the `doctave.yaml` file.

This allows you to:

* Decide on the order of the links
* Decide which links to show

## An example

As an example, below is this site's navigation config:

```
navigation:
  - path: docs/installing.md
  - path: docs/tutorial.md
  - path: docs/features
    children: "*"
  - path: docs/configuration.md
  - path: docs/contributors
    children: "*"
```

The `navigation` key expects a list of maps that describe a page, and any child pages. You can read
the above example as saying _"first include the installing page, then the tutorial page, then the
features directory and include all pages, next the configuration page, and finally the contributors
directory and all its children."_

The order in which links are included will be preserved in the navigation.

## Including a single page

In the simplest case, you can include a single page like so:

```
navigation:
  - path: docs/tutorial.md
```

## Including a directory

When including a directory, you have 3 options:

1. Only show the root of the directory and no other pages in the directory (children)

```
navigation:
  - path: docs/runbooks
```

2. Show the root link, and only _specific_ children

```
navigation:
  - path: docs/runbooks
    children:
      - path: docs/runbooks/deployment.md
```

3. Show the root link, and _all_ children

```
navigation:
  - path: docs/runbooks
    children: "*"
```

Note that the asterisk character has to be quoted in order to appease the YAML parser.
