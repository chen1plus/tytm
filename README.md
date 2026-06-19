# TyTM (Typora Theme Manager)

A theme package manager for Typora.

## Features

- [x] Add theme by ID (from registry)
- [x] Remove installed theme
- [x] List installed themes
- [x] Update installed themes

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
tytm add <THEME_ID>
```

Installs the specified theme from the built-in registry (`src/registry.json`).

Supported themes in the registry:
- `blackout` (Blackout)
- `blubook` (Blubook)
- `lapis` (Lapis)
- `maize` (Maize)
- `torillic` (Torillic)

### Update Themes

```sh
tytm update
```

Updates all installed themes to their latest version.

### Remove a Theme

```sh
tytm remove <THEME_ID>
# alias: tytm rm <THEME_ID>
```

Removes the specified theme and all its associated CSS styles and files.
