---
title: Tutorial
---

Tutorial
========

If you are new to Doctave, this tutorial will walk you through getting your site built and
deployed.

## Installation

First, make sure that you have installed Doctave locally. Follow the instructions in the
[installation guide](/installing).

To verify you have installed everything correctly, run the following:

```
$ doctave --version
Doctave CLI x.y.z
```

## Creating a new site

Creating a new documentation site can be done easily with the `doctave init` command:

```
$ doctave init
...
```

This will create a `docs/` directory in the root of your repository, and some pages for you to get
started with.

You'll also find a `doctave.yaml` in your project root now. Lets take a look at it.

```
# On Mac / Linux
$ cat doctave.yaml
---
title: "My project"



# On Windows
$ type doctave.yaml
---
title: "My project"

```

Currently, you only have the project's name mentioned. This title is shown on the page navigation,
and used as the HTML page title. You should change that to be the actual name of your project.

Now, you can run `doctave serve` to start the local webserver.

```
$ doctave serve

Doctave CLI | Serve
🚀 Starting development server...

Server running on http://0.0.0.0:4001/
```

And finally, go to [http://localhost:4001](http://localhost:4001) to view your site.

## Editing content

While `doctave serve` is running, you can edit the your documentation Markdown files, and you will
immediately see your page update. Try it! Open up `docs/README.md` in your favorite text editor,
and make a change. You should see the browser refresh and show your changes automatically. This way
you can quickly see what your changes look like.

If you are not familiar with Markdown syntax, or need a refresher, you can read our [Markdown
reference](/features/markdown) or check out [this
guide](https://guides.github.com/features/mastering-markdown/) by GitHub. Note that there are a few
different flavors of Markdown. Doctave supports all the "basics" Markdown features, as well as a few
"GitHub flavored Markdown" features - namely task lists and tables.

## Adding pages

To add a new page, all you need to do is add another Markdown file.

Let's say you want to add another page; a "How To Build" page. All you need to do is create a that
page inside your docs folder.

```
# On Mac / Linux
$ touch docs/building.md


# On Windows
$ echo.> docs\building.md
```

By default, Doctave assumes the title of the page is its title, but we may want to change that.
Let's add a _front matter block_ to the page. This is just a quick
[YAML](https://blog.stackpath.com/yaml/) snippet that gives Doctave some additional information
about the page.

Paste the following into the file you just created:

```
---
title: How to build
---

# How to build

...
```

Open up your browser, and you should now see a _"How to build"_ link in the sidebar. And if you
click it, you will be taken to the page.
