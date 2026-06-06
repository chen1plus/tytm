# TyTM (Typora Theme Manager)

A theme package manager for Typora.

## Current Features

- [x] Add a theme
- [x] Remove a theme
- [x] List installed themes
- [ ] Update manifest

## Installation

### macOS

Install via Homebrew:

```sh
brew tap chen1plus/tap
brew install chen1plus/tap/tytm
```

### Windows

Download `tytm.exe` from the release page.

### Build from Source

You can build and install it using Cargo:

```sh
cargo install --path .
```

## Usage

### List Installed Themes

```sh
tytm list
# alias: tytm ls
```

### Add a Theme

```sh
tytm add <URL>
```

The URL should point to a git repository or a zip file of the theme.

You can optionally specify the URL type (`git` or `zip`):

```sh
tytm add <URL> -u <git|zip>
```

### Remove a Theme

```sh
tytm remove <THEME>
# alias: tytm rm <THEME>
```

To remove specific sub-packages of a theme:

```sh
tytm remove <THEME> -s <SUB>
```
