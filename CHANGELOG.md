# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## v0.11.1 - 2026-08-23

### Fixed
- `run_with_msvc_env` again uses a temporary `.cmd` that calls `vcvarsall.bat x64_arm64` instead of inlining the command through PowerShell/`cmd /c`, which broke Windows paths on GitHub Actions (`'\' is not recognized`) during `package stage-native --msvc-env`.

### Technical
- Bumped workspace package version to **0.11.1**.

## v0.11.0 - 2026-08-23

### Added
- TUI Options overhaul: BIOS-style combo and textbox widgets, nested workflow steps, presets with TUI-only defaults, and `docs/tui-options.md`.
- Kernel form schema: `FieldKind::Preset`, `tui_default_value` / `tui_only` / `tui_combo_width` / `tui_nest`, and product hooks for applying presets.
- `build.run --cross-native` to stage cross-native targets buildable on the host.
- Windows MSVC DevShell re-exec helpers (`msvc` / non-Windows stub).

### Changed
- Options panel grouping and focus styling (content-sized combo/textbox highlights, terminal caret while editing).
- Diagnostics and native staging behavior from the cross-native / filedefs cleanup work.

### Technical
- Public `FieldSpec` gained new fields (breaking for external struct literals under Cargo 0.x rules).

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
