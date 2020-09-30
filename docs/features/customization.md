---
title: Customization
---

Customization
=============

Doctave provides a few ways to make your docs look like your own, namely setting a main color for
your site, and showing your logo in the top left of the page.

## Theme color

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

## Your logo

There are two steps to showing your logo on your site:

1. Place your logo under `docs/_assets` (read about custom assets [here](/features/assets))
2. Specify the filename of your logo in `doctave.yaml`

For exampe, with the following assets:

```
$ ls docs/_assets/
logo.png
```

And the following doctave.yaml:

```
---
title: Gonzo
logo: logo.png
```

Once you restart the Doctave server, you should see your logo at the top left of the page:

![Screenshot of your logo with the page title](/assets/logo-screenshot.png)

### Dimensions

Your logo will be cropped to a **45px by 45px** size. You should crop your logo to be close to that size
when deploying your site.

## Why don't you support themes?

While most generic static site generators support themes, Doctave has made the conscious decision
not to support custom themes. Doctave is meant to be a out-of-the-box and simple alternative for
building documentation sites, and adding theming support would make the tool more complicated than
necessary for what it is meant for.

If you are looking for more control around the look of your documentation, we recommend looking at
other great alternatives, such as [Mkdocs](https://www.mkdocs.org/),
[Jekyll](https://jekyllrb.com/), [Hugo](https://gohugo.io/), or [Zola](https://www.getzola.org/).
