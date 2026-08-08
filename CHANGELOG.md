# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## v0.10.0 - 2026-08-08

### Changed
- Merged Direct CLI and interactive TUI into a **single `drot` binary**. No subcommand on a TTY opens the TUI; a subcommand runs the CLI; `--help` / `--version` always print Direct meta (agents never need the TUI).
- Kept `drot_tui` as a **library only** (removed the separate `drot_tui` binary).
- Ship builds use `[profile.release]` (fat LTO, `opt-level = 3`, `panic = "abort"`). Local host `ensure-drot-dist` keeps `--profile dist` (thin LTO → `target/dist/`).
- Package Pipeline packs `cargo build -p drot --release` and stages from `target/release/`.

### Removed
- Separate `drot_tui` executable and dual-binary local dist stamp requirements on hosts.

### Technical
- Quality workflow clippy/test cover `drot` + `drot_tui` lib in one path.
- Docs (README, AGENTS, architecture) describe one-binary dispatch.
