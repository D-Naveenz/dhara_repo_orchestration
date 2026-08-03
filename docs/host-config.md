# Host `dhara.config.toml` and secrets

How DROT reads a host product repository (for example [dhara_storage](https://github.com/D-Naveenz/dhara_storage)).

## File roles

| File | Tracked? | Role |
|------|----------|------|
| `dhara.config.toml` | Yes | Shared product metadata, NuGet **feed** URL, CI pack/verify project paths, RID → rustc map |
| `.env.example` | Yes | Empty local secret **names** (`CARGO_REGISTRY_TOKEN`, `NUGET_API_KEY`) |
| `.env.local` | No | Secret **values** for local `package publish` / `release run` |

Activation (CLI and TUI) scaffolds missing config / env after the repository path is confirmed: skeleton `dhara.config.toml` when absent; `.env.local` from `.env.example` or the same two empty keys.

Trusted Publishing (OIDC) works only in GitHub Actions. Local publish always needs `.env.local` (or the process environment).

## Config ownership

**In `dhara.config.toml` (shared / operator):**

- `[versions].workspace` — single product version
- `[product]` — `authors`, `repository_url`, `project_url`, optional `license`
- `[nuget].source` — default NuGet feed (not package metadata)
- `[ci]` — `package_project` (native-bearing primary), `managed_package_projects`, smoke/tests projects, native + smoke RIDs
- `[targets.rust_targets]` — RID → rustc triple

**In each project file (csproj / Cargo.toml):**

- Package id / crate name, description, tags, README, icon, and other package-only fields

Activation drift syncs **only** shared fields: workspace version plus product authors/URLs/license into listed package projects and Cargo `[workspace.package]`. It does **not** overwrite per-package PackageId/Description/tags/icon/README from config.

## Pack / publish

- `package pack` packs `ci.package_project` (with staged natives) then each `ci.managed_package_projects` entry into the same nuget output folder.
- Local NuGet push uses hardcoded `NUGET_API_KEY`; crates.io uses hardcoded `CARGO_REGISTRY_TOKEN`.
- Host CI may publish without DROT (composite Actions + GitHub Environments). Those Environment names live in workflow YAML only — never in `dhara.config.toml`.
