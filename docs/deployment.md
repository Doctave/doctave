---
title: Deployment
---

Deployment
==========

Doctave builds a static site bundle into the `site` directory which is fully self contained. You can
take the output of `doctave build --release` and deploy it in any way you see fit.

{% info :point_right: Deploying under a custom path %}

By default Doctave assumes the deployed page is accessible under the root url. E.g.
`wwww.example.com`.

If you want to deploy under a subdirectory, such as `www.example.com/docs`, you
have to set the `base_path` in your `doctave.yaml` file. This will make Doctave generate URLs
relative to e.g `/docs` instead of the plain root url `/`.

Read more [here](/configuration#base_path-5).

{% end %}

Below is a walkthrough on how to publish your docs on Github Pages. We will be adding more tutorials
for other hosting options over time.

## Github Pages

This guide assumes you have:

1. Hosted your repository on Github
2. Enabled Github Pages in your repository's settings

### Disable Jekyll builds

First, you need to tell Github not to use Jekyll to build your site. You do this by including a
`.nojekyll` file in the root of site. 

In Doctave, you do this by adding the file under `docs/_include`.

On Mac/Linux:
```
$ touch docs/_include/.nojekyll
```

On Windows:
```
$ echo.> docs\_include\.nojekyll
```

### Custom domain name (optional)

If you have a custom domain, Github requires you to create a `CNAME` file that describes your domain
in the root of your site. Just like with the `.nojekyll` file, you need to place this under
`docs/_include`.

### Install gh-pages

There are a few ways to push your site to Github. One way, which we will follow here, is to use a
`gh-pages` branch to publish the site. This means you build the site into a separate branch, commit
only the `site` folder in that branch, not your other source code, and push it to Github.

Luckily, there is a handy command line tool, [gh-pages](https://www.npmjs.com/package/gh-pages)
that takes care of all of that for you. All you need to do, is run a single command, and your site
will be published.


{% warning :warning: Note about gh-pages %}
At the time of writing, `gh-pages@3.1.0` does not work for projects without a package.json
file. This is why this guide recommends using `3.0.0`. Read more in [this
issue](https://github.com/tschaub/gh-pages/issues/354).
{% end %}

```
npm install -g gh-pages@3.0.0
```

### Build your site

Next, build your site in release mode. This strips away some development-only dependencies:

```
$ doctave build --release
```

Your site should now be ready in the `site` directory.

### Deploy

All that is left to do, is run the `gh-pages` command:

```
$ gh-pages -d site
Published
```
