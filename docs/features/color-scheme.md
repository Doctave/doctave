---
title: Color scheme
---

Color scheme
============

While Doctave by design does not support advanced themes, it supports changing the default blue
color scheme via the doctave.yaml. Here's an example:

```yaml
---
title: Doctave CLI
colors:
  main: "#5f658a"
```

Setting the `colors.main` key to a HEX color value will compute a new color scheme. For the dark
theme, Doctave will compute a lighter color based on the provided color in order to provide better
contrast against a black background.

## Why don't you support themes?

While most generic static site generators support themes, Doctave has made the conscious decision
not to support custom themes. Doctave is meant to be a out-of-the-box and simple alternative for
building documentation sites, and adding theming support would make the tool more complicated than
necessary for what it is meant for.

If you are looking for more control around the look of your documentation, we recommend looking at
other great alternatives, such as [Mkdocs](https://www.mkdocs.org/),
[Jekyll](https://jekyllrb.com/), [Hugo](https://gohugo.io/), or [Zola](https://www.getzola.org/).
