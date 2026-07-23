# DROT — Dhara Repository Orchestration Tool

[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)

Operator tooling for [Dhara Storage](https://github.com/D-Naveenz/dhara_storage) workspaces (and other Dhara repositories via product plugins).

| Binary | Audience |
|--------|----------|
| **`drot`** | Direct CLI for CI, scripts, and agents |
| **`drot_tui`** | Interactive TUI for human developers |

Version authority is this workspace's `[workspace.package].version` in `Cargo.toml` (currently **0.9.12**). Host repos (such as `dhara_storage`) pin a commit via git submodule at `tooling/drot` and do not co-own the tool version.

## Layout

```
src/
  drot_kernel/         # Dhara framework: paths, config, logging, host APIs, root args
  drot_dhara_storage/  # Storage product plugin (commands, ops, filedefs)
    package/           # TrID archives copied beside the CLI at build
    data/              # MIME/extension catalogs (compile-time)
  drot/                # CLI binary
  drot_tui/            # TUI library + binary
```

Hosts register plugins at startup via `drot_dhara_storage::plugins()` / `install_hooks()`. Dynamic cdylib discovery is deferred; see kernel `bootstrap` / `product` modules for the compile-time seam.

## Local build

```bash
cargo build -p drot --profile dist
cargo build -p drot_tui
cargo test -p drot -p drot_kernel -p drot_dhara_storage
```

## CI artifacts

On PR and merge to `main`, workflows upload architecture-specific CLI packages:

- `drot-windows-x64`
- `drot-linux-x64`

Consumers (e.g. `dhara_storage`) download the artifact matching the runner OS for the pinned submodule commit — they never compile DROT in their CI.

## License

Apache-2.0
