---
title: Installing
---

Installing Doctave
==================

There are a few installation options for Doctave. If you would like another installation option,
please open an issue for it.

### Precompiled binaries

Doctave provides precompiled binaries for Mac, Linux, and Windows, which you can download from the
[latest release page](https://github.com/Doctave/doctave/releases/latest).

### Homebrew

Doctave maintains its own [homebrew tap](https://github.com/Doctave/homebrew-doctave), and you can
install Doctave via the following command:

```
$ brew install doctave/doctave/doctave
```
This will take a few minutes as Doctave is compiled from scratch for your machine.
### Cargo (Rust package manager)

You can also use the Rust package manager, Cargo, to install Doctave. Currently Doctave is not
listed on crates.io, but you can install it directly from GitHub:

```
$ cargo install --git https://github.com/Doctave/doctave --tag 0.4.0
```
