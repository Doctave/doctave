---
title: Custom head tag
---

Custom head tag
===============

You may want to sometimes include things like analytics trackers, or other Javascript  snippets on
your site. Doctave allows you to inject some HTML into the `<head>` tag of every page to help with
this.

## How does it work?

The mechanism is very simple. All you have to do, is create a file called
`docs/_include/_head.html`.

Doctave will pick this up, and inject the contents of it inside every page's `<head>` tag.

The contents of the file are not verified in any way, so syntax issues can impact the rest of the
site dramatically.

## A note on using this feature

This feature technically opens the door for users to do heavy customization of their sites. However,
Doctave is an opinionated tool and does not officially support customization features like themes.
If you plan to use this feature to customize Doctave, be aware that this is not officially something
Doctave endorses, and _your code may break in future releases_.

This feature is meant for adding e.g. Google Analytics or Intercom integrations, not customize the
look and feel of the site.

If you need more customization options, we recommend looking at other static site generators.
