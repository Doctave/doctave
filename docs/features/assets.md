---
title: Custom assets
---

Custom assets
=============

If you have images or other files besides Markdown files that you wish to include in your project,
you can place them in a `docs/_include` directory. Doctave will copy all files from that directory
into the final site bundle, and place in the root of the directory.

Here's a couple examples:

## Images

A `docs/_include/assets/cat.jpg` file would get a final path of `/assets/cat.jpg`, which you can
reference in your documentation as:

```markdown
![my lovely cat](/assets/cat.jpg)
```

## Favicon

You can include a custom favicon by placing a `favicon.ico` into the `docs/_include` directory.
