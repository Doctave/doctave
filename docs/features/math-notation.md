---
title: Mathematical Notation
---

Mathematical Notation
=====================

Doctave will render mathematical formulas written in [Tex](https://en.wikipedia.org/wiki/TeX) into nicely typeset
notation. This feature is powered by [KaTex](https://katex.org/).

## Example

To place a formula onto your page, create a code block and specify `math` as its language:

~~~
```math
x^2 - 5x + 6 = 0 \\
(x-2)(x-3)=0 \\
\textrm{then either }x=2 \,or\,x=3
```
~~~

This will get rendered as follows:

```math
x^2 - 5x + 6 = 0 \\
(x-2)(x-3)=0 \\
\textrm{then either }x=2 \,or\,x=3
```

Or for something more complicated:

~~~
```math
\left( \sum_{k=1}^n a_k b_k \right)^2 \leq
\left( \sum_{k=1}^n a_k^2 \right)
\left( \sum_{k=1}^n b_k^2 \right)
```
~~~

We get:

```math
\left( \sum_{k=1}^n a_k b_k \right)^2 \leq
\left( \sum_{k=1}^n a_k^2 \right)
\left( \sum_{k=1}^n b_k^2 \right)
```

## Where can I learn more?

Here are some useful links if you are not familiar with TeX or KaTeX specifically.

* [A gentle introduction to TeX](https://www.texlive.info/CTAN/info/gentle/gentle.pdf)
* [Supported functions in KaTeX](https://katex.org/docs/supported.html)