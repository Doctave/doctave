---
title: Tutorial
---

Tutorial
========

If you are new to Doctave, this tutorial will walk you through getting your site built and
deployed.

# Installation

First, make sure that you have installed Doctave locally. Follow the instructions in the
[installation guide](/installing).

To verify you have installed everything correctly, run the following:

```
$ doctave --version
Doctave CLI x.y.z
```

# Creating a new site

Creating a new documentation site can be done easily with the `doctave init` command:

```
$ doctave init
```

This will create a `docs/` directory in the root of your repository, and some pages for you to get
started with.

You'll also find a `doctave.yaml` in your project root now. Lets take a look at it.

```
$ cat doctave.yaml
---
title: "My project"

```

Currently, you only have the project's name mentioned. *You should change that to be the actual name
of your project.
