# DROT

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

**Dhara Repository Orchestration Tool** — operator CLI and TUI for Dhara workspaces (config, packaging, file definitions, release flows).

Host repositories such as [dhara_storage][dhara-storage] pin this project as a git submodule and do **not** co-own the tool version. Version authority is this workspace’s `[workspace.package].version` in `Cargo.toml`.

One binary: **`drot`**. No subcommand on a TTY opens the interactive TUI; a subcommand runs the Direct CLI; `--help` lists commands without opening the TUI (for agents and scripts).

## Prerequisites

- Rust **stable** toolchain
- Access to the host workspace you are operating on (for the storage product extension)

## Install / build

From this repository root:

```bash
cargo build -p drot --profile dist
```

Hosts enable the default Cargo feature `extension-dhara-storage` (links `drot_dhara_storage`). Build with `--no-default-features` for a kernel-only binary (base commands disabled until an extension is linked).

Run tests:

```bash
cargo test -p drot -p drot_kernel -p drot_dhara_storage -p drot_tui
```

Consumers often download CI artifacts (`drot-windows-x64`, `drot-linux-x64`) for the pinned submodule commit instead of compiling DROT in their own CI.

## Usage

### 1. Point at a host repository

Typical invocations from a host (example: dhara_storage) use `-r` / `--repository` after the binary is on your `PATH` or under `target/dist/`.

### 2. Common operator flows

Exact subcommands depend on the linked product extension (for example `drot_dhara_storage`). Typical areas:

- Config activation and environment scaffolding
- Native staging / package verify
- File definition sync and inspect
- Release dry-runs

Prefer this repo’s [AGENTS.md](AGENTS.md) and the host’s scripts for the exact commands that host expects.

```bash
drot --help
drot -r <host-repo> --yes quality run
```

### 3. TUI

Interactive three-panel shell: Tasks tree, tabbed center (Info / Options / Troubleshooting / System), Actions (progress + Run/Cancel).

From a host such as dhara_storage, prefer the host’s `run-drot` script (git-stamps `target/dist/drot` against this checkout’s `HEAD`). With no subcommand it opens the TUI; pass a subcommand or `--help` for the Direct CLI.

From this repository root:

```bash
cargo run -p drot --profile dist -- -r <host-repo>
```

## Related

- Host product: [dhara_storage][dhara-storage]
- Tool agent router: [AGENTS.md](AGENTS.md)
- Deep reference: [docs/](docs/README.md)
- Orchestration repo: [dhara_repo_orchestration][orch]

## License

Apache-2.0.

[dhara-storage]: https://github.com/D-Naveenz/dhara_storage
[orch]: https://github.com/D-Naveenz/dhara_repo_orchestration
