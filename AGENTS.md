# AGENTS.md

Read this file before large changes in the DROT (Dhara Repository Orchestration Tool) repository. It is the durable product note and AI/dev router for this workspace.

Host products (for example [dhara_storage](https://github.com/D-Naveenz/dhara_storage)) pin this repo as a **git submodule**. Do **not** keep DROT-deep docs, rules, or architecture essays in the host — they belong here.

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

- One tool version authority (`[workspace.package].version` in this repo)
- Direct CLI for CI/agents; TUI for developers
- Kernel framework + product plugin (`drot_dhara_storage`) so hosts stay thin
- Progress and audit logging that work in both modes

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
| `crates/drot_kernel` | Framework — commands, forms, runner, interactive, logging, progress |
| `crates/drot_dhara_storage` | Storage product plugin — registry, ops, filedefs |
| `crates/drot` | Direct CLI host |
| `crates/drot_tui` | Interactive TUI host |
| `docs/**` | Deep reference (logging, TUI progress, architecture) |

Deep reference: [docs/README.md](docs/README.md).

---

## Local commands

From this repository root:

```bash
cargo build -p drot -p drot_tui --profile dist
cargo test -p drot -p drot_kernel -p drot_dhara_storage -p drot_tui
cargo run -p drot -- -r <host-repo> --yes quality run
cargo run -p drot_tui --profile dist
```

When developed as a submodule under a host, hosts typically wrap builds with scripts that version-gate `target/dist/` against this `Cargo.toml`.

---

## CI / pack

Orchestration CI packs `drot` / `drot_tui` artifacts per OS. Hosts download by **submodule SHA** (not by guessing tool version alone).

---

## Guardrails

- Keep DROT docs and `.cursor/rules` in **this** repository; hosts should link here.
- Do not invent a second tool version in host `dhara.config.toml`.
- Breaking changes are acceptable pre-1.0; prefer clean cuts over parallel APIs.
- Prefer Windows as the primary developer workstation for TUI verification.
