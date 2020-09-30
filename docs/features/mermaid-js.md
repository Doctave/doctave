---
title: Mermaid.js diagrams
---

Mermaid.js diagrams
===================

Doctave supports [Mermaid JS](https://mermaid-js.github.io/) diagrams out of the box. All you need
to do is specify the `mermaid` language for your codeblock, and Doctave will render the diagram for
you.

Diagram types supported include

* Pie chards
* Sequence diagrams
* Flowcharts
* Class diagrams
* State diagrams

## Basic example

Specify the mermaid as your code block language:

~~~
```mermaid
graph TD;
    A-->B;
    A-->C;
    B-->D;
    C-->D;

```
~~~

and see your graph rendered as follows:

```mermaid
graph TD;
    A-->B;
    A-->C;
    B-->D;
    C-->D;

```

## Learning Mermaid.js

If you are not familiar with Mermaid JS, we suggest taking a look at their
[tutorials](https://mermaid-js.github.io/mermaid/diagrams-and-syntax-and-examples/n00b-syntaxReference.html)
and playing around with their [live editor](https://mermaid-js.github.io/mermaid-live-editor)

## Further examples

### Pie chart
~~~
```mermaid
pie title Favorite pie flavor
         "Lemon" : 2
         "Apple" : 3
         "Blueberry" : 5.6
```
~~~

```mermaid
pie title Favorite pie flavor
         "Lemon" : 2
         "Apple" : 3
         "Blueberry" : 5.6
```

### Sequence diagram

~~~
```mermaid
sequenceDiagram
    Alice ->> Bob: Hello Bob, how are you?
    Bob-->>John: How about you John?
    Bob--x Alice: I am good thanks!
    Bob-x John: I am good thanks!
    Note right of John: Bob thinks a long<br/>long time, so long<br/>that the text does<br/>not fit on a row.

    Bob-->Alice: Checking with John...
    Alice->John: Yes... John, how are you?
```
~~~

```mermaid
sequenceDiagram
    Alice ->> Bob: Hello Bob, how are you?
    Bob-->>John: How about you John?
    Bob--x Alice: I am good thanks!
    Bob-x John: I am good thanks!
    Note right of John: Bob thinks a long<br/>long time, so long<br/>that the text does<br/>not fit on a row.

    Bob-->Alice: Checking with John...
    Alice->John: Yes... John, how are you?
```
[Source](https://mermaid-js.github.io/mermaid/diagrams-and-syntax-and-examples/examples.html#basic-sequence-diagram).


### State diagram

~~~
```mermaid
stateDiagram-v2
    [*] --> Still
    Still --> [*]

    Still --> Moving
    Moving --> Still
    Moving --> Crash
    Crash --> [*]
```
~~~

```mermaid
stateDiagram-v2
    [*] --> Still
    Still --> [*]

    Still --> Moving
    Moving --> Still
    Moving --> Crash
    Crash --> [*]
```

[Source](https://mermaid-js.github.io/mermaid/diagrams-and-syntax-and-examples/stateDiagram.html).
