# AGENTS.md

Read this file before large changes in the DROT (Dhara Repository Orchestration Tool) repository. It is the durable product note and AI/dev router for this workspace.

Host products (for example [dhara_storage](https://github.com/D-Naveenz/dhara_storage)) pin this repo as a **git submodule**. Do **not** keep DROT-deep docs, rules, or architecture essays in the host — they belong here. When pinned under a host, also consider the host `AGENTS.md` and host `docs/` for product CI and runtime — do not paste host essays here.

## Human vs AI docs

| Surface | Audience | Role |
|---------|----------|------|
| `README.md` | Humans | What / why / how to use |
| This file (`AGENTS.md`) | Humans + AI | Ambition, architecture, commands, CI, guardrails |
| `docs/**` | Implementers | Logging, TUI progress, crate DAG |

**Agents:** follow the global **`project-docs`** skill, then [`.cursor/rules/project-docs.mdc`](.cursor/rules/project-docs.mdc) for README / AGENTS / `docs/`. For source **documentation comments**, headers, and why-comments, follow **`inline-code-docs`**. Logging changes: [`.cursor/rules/logging.mdc`](.cursor/rules/logging.mdc) + [docs/logging.md](docs/logging.md).

---

## Product intent

### Ambition

Cross-product **operator** CLI and TUI for Dhara workspaces: config activation, packaging, file definitions, quality, and release flows — with the same mental model for scripts and interactive use.

### Goals

- One tool version authority (`[workspace.package].version` in this repo’s `Cargo.toml`)
- One `drot` binary: Direct CLI for CI/agents; TUI when launched with no subcommand on a TTY
- Kernel framework + one compile-time **product extension** (default: `drot_dhara_storage`) so hosts stay thin
- Progress and audit logging that work in both modes (session starts at activation)

### Host vs this repo

| Concern | Owner |
|---------|--------|
| Tool version, crates, TUI/CLI behavior, DROT docs/rules | **This repo** |
| Product runtime (`dhara_storage`), NuGet, storage CI maps | **Host** (e.g. dhara_storage) |
| Submodule gitlink pin | Host |

---

## Architecture map

| Path | Role |
|------|------|
| `crates/drot_kernel` | Framework — base commands, registry, forms, runner, interactive, logging, progress |
| `crates/drot_dhara_storage` | Storage product **extension** — upserts handlers, ops, filedefs |
| `crates/drot` | Single binary host — Direct CLI + interactive TUI (`extension-dhara-storage` feature, default on) |
| `crates/drot_tui` | TUI library used by `drot` (no separate binary) |
| `docs/**` | Deep reference (logging, TUI progress, architecture) |

Hosts link **exactly one** extension via Cargo features at compile time. Kernel registers base command stubs; the extension adds product commands and fills handlers. Commands without a handler or with `is_disabled` are listed but warn on execute.

Deep reference: [docs/README.md](docs/README.md).

---

## Local commands

From this repository root (standalone checkout):

```bash
cargo build -p drot --profile dist
cargo test -p drot -p drot_kernel -p drot_dhara_storage -p drot_tui
cargo run -p drot -- -r <host-repo> --yes quality run
cargo run -p drot --profile dist -- -r <host-repo>
```

Binary lands in **this** repo’s `target/dist/drot` (`.exe` on Windows). No subcommand on a TTY opens the TUI; `--help` lists commands.

### Host submodule layout (agents)

When this repo is pinned under a host (e.g. `dhara_storage/tooling/drot`):

| Concern | Path |
|---------|------|
| **Source** (edit here) | `tooling/drot/` (this checkout) |
| **Run / rebuild** | Host scripts such as `./tooling/scripts/run-drot.ps1` / `ensure-drot-dist` |
| **Binary + git stamp** | Host `<repo>/target/dist/drot` and `.drot-git-rev` — **not** under the submodule |

Host wrappers set `CARGO_TARGET_DIR=<host>/target` and build `--manifest-path tooling/drot/Cargo.toml -p drot --profile dist`. Do **not** search the submodule tree for `drot.exe`. When spawning subagents for DROT work, pass both the source root (`tooling/drot`) and the host artifact path (`target/dist`).

---

## CI / pack

Orchestration CI splits **Quality** (`quality.yml` on PRs; skips Dependabot → `development`) from **Package Pipeline** (`package-pipeline.yml` on `push` to `main` + `workflow_dispatch` with optional `checkout_sha`). Hosts download `drot-*-x64` artifacts by **submodule SHA**. Hosts may run `ensure-drot-artifacts` when the pin has no successful pack. Branch flow: `feature` → `development` → `main` (never auto-merge into `main`).

---

## Guardrails

- Keep DROT docs and `.cursor/rules` in **this** repository; hosts should link here.
- Do not invent a second tool version in host `dhara.config.toml` — tool version is `[workspace.package].version` in this `Cargo.toml`.
- Host package-specific NuGet/Cargo metadata stays in each csproj / `Cargo.toml`; config holds shared `[product]`, slim `[nuget].source`, and `[ci]` paths — see [docs/host-config.md](docs/host-config.md).
- Prefer clean-cut breaking changes; decide in plan / follow plan / ask when unclear — see [`.cursor/rules/breaking-changes.mdc`](.cursor/rules/breaking-changes.mdc).
- Prefer Windows as the primary developer workstation for TUI verification.
