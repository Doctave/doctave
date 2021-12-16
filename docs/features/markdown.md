---
title: Markdown syntax
---

Markdown syntax
===============

This document walks you through all the various Markdown features and associated syntaxes.

#[I don't exist](/nothing/here)

## Headings

All heading types are supported.

```
// Underline style
H1
==

// Hash style
# H1
## H2
### H3
#### H4
##### H5
###### H6
```

Note that headings smaller than H3 will not show up on the right-side navigation. Also, the first
heading on the page will be emphasized, like the "Markdown syntax" title on this page.

## Emphasis

```
Both **bold**, _italics_, and ~~strikethrough~~ are supported.
```

Both **bold**, _italics_, and ~~strikethrough~~ are supported.

## Lists

### Unordered lists

```
* Mary
* Had
* A little
    * Lamb
```

* Mary
* Had
* A little
    * Lamb

### Ordered lists

```
1. Mary
2. Had
3. A little
    1. Lamb
```

1. Mary
2. Had
3. A little
    1. Lamb

## Links

```
[Doctave](https://doctave.com)
```

[Doctave](https://doctave.com)

## Images

```
![A random image](https://picsum.photos/600/400)
```

![A random image](https://picsum.photos/600/400)

## Emojis

You can use [GitHub style shortcodes](https://emojipedia.org/shortcodes/) for emojis.

This means you can write `:bell:` and Doctave will render is as :bell:. These work particularly well
with callouts (see below):

{% info :zap:  I want your attention %}
This is important!
{% end %}

## Callouts

You can bring attention to sections using callouts.


### Basic usage
```
{% info This is the title %}

And this is the content!

So much space for activities!

{% end %}
```

Gives you:

{% info This is the title %}

And this is the content!

So much space for activities!

{% end %}

### Without a title

```
{% info %}

You can skip the title

{% end %}
```

Gives you:

{% info %}

You can skip the title

{% end %}

### Types

Instead of `info`, you can use `success`, `warning`, or `error` for different colored callouts:

{% success A success message %}
This is positive news
{% end %}

{% warning A warning message %}
You should use caution
{% end %}

{% error An error message %}
Something went wrong
{% end %}

## Quotes

```
> It's true, because it's a quote
```

> It's true, because it's a quote

## Code

### Inline

```
Use `backticks for inline code snippets`.
```

Use `backticks for inline code snippets`.

### Block

Either use three backticks
~~~
```
For().your().code()
```
~~~

To achieve:

````
For().your().code()
````

Or indent your code with 4 spaces:
````

    For().your().code()

````

For the same effect:

    For().your().code()

You can specify the language for syntax highlighting using the backticks syntax.

~~~
```ruby
def initialize(table_name_singularised)
    @table = table_name_singularised.to_s.pluralize
end
```
~~~

```ruby
def initialize(table_name_singularised)
    @table = table_name_singularised.to_s.pluralize
end
```

## Task Lists

```
- [ ] This is a list of todos
- [x] This is a completed item
- [ ] This is an uncompleted item
```

- [ ] This is a list of todos
- [x] This is a completed item
- [ ] This is an uncompleted item

## Tables

```
This is a heading              | This is another heading  |
-------------------------------|--------------------------|
This is content for a columns  | This is **bold**         |
You can have more rows         | And more columns         |
```

This is a heading              | This is another heading  |
-------------------------------|--------------------------|
This is content for a columns  | This is **bold**         |
You can have more rows         | And more columns         |
