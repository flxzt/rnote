# Contributing to Rnote

This document provides a guideline how to contribute to Rnote.

# Maintainers

The current core maintainers of the project are:
- @flxzt : Original Author and Maintainer
- @Doublonmousse : Maintainer
- @Kneemund : Maintainer

# Contribute Bug Reports & Feature Requests

File a bug report or feature request directly through the github web UI
by clicking on `Repository->Issues->New Issue`.
Choose between the different existing templates and **please** fill them out completely as requested.

# Contribute Translations

A great way to contribute to the project without writing code is adding a new or start maintaining an existing
translation language.
The translation files are located in `crates/rnote-ui/po/`.
Creating translations for new languages or updating existing ones can be done in multiple ways:
- Take the `rnote.pot` file and generate a new `.po` translation file from it, for example with "Poedit".
    Add the new translation language to `LINGUAS` and submit a PR with the changed files.
- Use [weblate](https://hosted.weblate.org/projects/rnote/repo/) for an easy way to translate in the browser
    without having to deal with git.

# Contribute Code

Code changes and additions need to adhere to the code checks outlined in chapter [#Code Checks](#code-checks).
All changes and additions should go through a PR->Review cycle.
The core maintainers can also push directly to main but should only do that in case of trivial changes and fixes.
The CI must run successfully to get a change merged.
Ideally the optional lint step does not report any warnings.
But because new lints can appear on new clippy versions this is not mandatory.
Please add a short description outlining the changes and the reasons for them.
When adding new features and/or changes in the UI some screenshots or screen captures would be nice.
When it fixes a specific issue, the description should reference the to-be-fixed issue with `fixes #<num>`.

## Usage of LLM/Gen-AI

**Vibe-coded contributions that were primarily implemented by an LLM will not be accepted.**  
There is very limited value in having an external contributor as an intermediary between a maintainer and an LLM/Gen-AI
model.
It's also worth emphasizing that all contributions are reviewed by humans with limited available time.
LLM-driven contributions can easily create a dangerous imbalance where a maintainer invests more time into a change than
the contributor themselves.

Only consider to open a change with LLM assistance after careful consideration for the time spent needed for a review.
Always disclose for which part of a change LLM/Gen-AI tools were used in detail.
Do not let the LLM generate descriptions, comments or other text intended for human consumption.
If maintainers suspect a review is simply forwarded to an LLM they are free to abort the review and deny the change
request without further reason.
Do not include trailers like “Co-authored-by:” or “Assisted-by:” in commit messages, since they serve as free
advertising for LLM/Gen-AI companies.

**You need to own the code fully and you must be confident to be able defend your choices for any line of changed
code.**
Otherwise maintainers are free to deny a change request without further reason.

# Code Checks

## Pre-Commit hook

When the developer runs the "prerequisite" recipe after checking out the repository a pre-commit hook is installed to
ensure code checks already at the time a change is committed.
Take a look at the [pre-commit.hook](hooks/pre-commit.hook) file to see what the hook checks in detail.

## Formatting

For formatting `rustfmt` is used. It picks up the formatting configuration file `rustfmt.toml`.
To check the formatting run:

```bash
just fmt-check
```

And to directly apply run:

```bash
just fmt
```

The formatting is also checked in the CI and applied formatting is a prerequisite for merging additional or changed
code.

## Lints

For linting rust code `clippy` is used.
To lint the codebase run:

```bash
just lint
```

If carefully considered clippy warnings can also be disabled in code by using `#[allow(clippy::\<lint-name\>)].
However this must be justified.

## Tests

Unit tests are added throughout the codebase.
To run them, install [cargo-nextest](https://nexte.st/).
Additionally, the style and correctness of other data/auxiliary files like the `.desktop` or `metainfo.xml` AppData
definition file is checked as well.
To execute all tests, run:

```bash
just test
```

### Adding unit-tests

Just like in any other rust crate, tests can be added by declaring a tests module prefixed with the #[cfg(test)]
attribute, then adding test functions prefixed by the #[test] attribute.
Tests should be as closely coupled to the code they target as reasonably possible and in most cases should reside in the
same source file.

# Supported Platforms

The application currently supports the following platforms:

## Linux

Rnote is mainly developed for Linux and integrates best with the Gnome desktop environment.
The application should nonetheless function properly regardless of which DE, compositor or distribution is used.
In addition the focus for development and testing is on Wayland, at this point X11 has a lot of issues and
inconsistencies especially with regards to pen input which is an integral part of the application.
This is why X11 is now considered unsupported.
For more details on how to build the application on Linux either natively or as flatpak see:
[BUILDING.md](./BUILDING.md).

## macOS

The application is also bundled for macOS, @dehesselle is active in issues that affect the app bundle.
For more details on how to build the application on macOS see:
[docs/build-macos.md](./docs/build-macos.md).

## Windows

For windows `msys/mingw64` is used as the development and build environment.
For the installer "Inno Setup" is used.
It should always be ensured that the app will build in `msys/mingw64`, however tight integration with the Windows OS is
not a priority.
For more details on how to build the application and the installer on Windows see:
[docs/build-win.md](./docs/build-win.md).


# Dependencies

Rust dependencies are declared in the root workspace `Cargo.toml`, or if crate-specific, in the individual crate's
`Cargo.toml` configuration files.
The generated `Cargo.lock` file pins the dependencies to specific versions and is checked in.
All non-rust dependencies are declared in the root `meson.build` file.
For example, you will find declarations for C dependencies like `glib` and `gtk4`.

# Documentation

The `rnote-compose` and `rnote-engine` crates should be treated as stand-alone libraries and should contain at least
a bit of documentation for their features and functionality.
The `rnote-cli` and `rnote-ui` crates are "consumer" crates and especially the UI contains a ton of boilerplate code so
in there documentation is not so critical.
However especially Gtk quirks and workarounds should always be documented in code.

# Architecture

Architectural decisions are documented in [docs/arch.md](./docs/arch.md)
