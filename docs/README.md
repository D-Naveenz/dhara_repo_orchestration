# DROT technical reference

Deep reference for the Dhara Repository Orchestration Tool. Human onboarding stays in [README.md](../README.md). Agent/router notes stay in [AGENTS.md](../AGENTS.md).

| Doc | Audience | Topic |
|-----|----------|-------|
| [Architecture](architecture.md) | Agents, contributors | Crate DAG, TUI layout, path resolution |
| [Host config and secrets](host-config.md) | Operators, agents | `dhara.config.toml` ownership, `.env.local`, activation scaffold |
| [Logging conventions](logging.md) | Operators, agents | Audit tiers, session lifecycle, TrID phase lines |
| [TUI operation progress](tui-progress.md) | Operators, agents | Progress bar lifecycle, discover→commit→tick |
| [Filedefs sluice policy](filedefs-sluice.md) | Agents, contributors | Extension seed levels, split-archive tokens, CAD/EDA exclusions |

Host-product docs (DSFD format, storage CI, NuGet packaging) live in the host repo — for storage see [dhara_storage/docs](https://github.com/D-Naveenz/dhara_storage/tree/main/docs).
