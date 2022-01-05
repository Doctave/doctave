Changelog
=========

Starting from version 0.4.0, Doctave will maintain this changelog to describe changes in each release.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1](https://github.com/Doctave/doctave/releases/tag/0.4.1) 2022-01-05

### Fixed

- Fix CSS class name collision with some Prism grammars [#34](https://github.com/Doctave/doctave/issues/34)

## [0.4.0](https://github.com/Doctave/doctave/releases/tag/0.4.0) 2021-12-29

This was the biggest release since the initial introduction of Doctave, adding a number of interesting new features.

* [GitHub Milestone](https://github.com/Doctave/doctave/milestone/1)

### Added

- Adds support for rendering mathematical notation [#14](https://github.com/Doctave/doctave/issues/14)
- Broken links checking during build process [#13](https://github.com/Doctave/doctave/issues/13)
- Expanded supported languages for syntax highlighting [#24](https://github.com/Doctave/doctave/issues/24)

### Changed

- Anchor links no longer contain a link index in the URL hash part [#15](https://github.com/Doctave/doctave/issues/15)
- Updated Mermaid.JS version [#17](https://github.com/Doctave/doctave/issues/17)

### Fixed

- Build failure when including the root path in the navigation hierarchy [#18](https://github.com/Doctave/doctave/issues/18)
  ([@datdenkikniet](https://github.com/datdenkikniet))
- Right side navigation not including text from code blocks in headings [#15](https://github.com/Doctave/doctave/issues/15)