# AGENTS.md

- Rust CLI crate, single binary `tytm`; source entrypoint is `src/main.rs`.
- Commands are `add`, `remove`/`rm`, `list`/`ls`, and `update`; `update` is stubbed with `todo!()` and will panic if run.
- Debug builds use repo-local paths under `./debug/Typora/themes` and `./debug/tytm/manifest.json`; release builds use the OS data directory.
- `Manifest` only accepts `manifest_version = "1"`; load falls back to an empty manifest on missing, corrupt, or mismatched files.
- `add` accepts git repos or zip files; if `-u` is omitted it infers git from `.git` and zip from `.zip`.
- `remove` matches either the manifest key or the theme’s `*.css` filename; `-s/--sub` removes only listed files and keeps the manifest entry if any files remain.
- `list` auto-deletes manifest entries whose tracked files no longer exist, then saves the manifest.
- Tests are integration tests in `tests/integration_test.rs`; they expect the binary to run in a temp working directory and will create the debug tree there.
- Run the focused suite with `cargo test --test integration_test`.
- CI runs `cargo check` and `cargo test` only.
- There is no repo-local `AGENTS.md`, `CLAUDE.md`, cursor rules, or OpenCode config to mirror.
