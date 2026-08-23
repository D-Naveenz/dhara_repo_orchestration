# Filedefs sluice policy

TrID definitions are reduced in `drot_dhara_storage` before embedding into `filedefs.dat`. The sluice admits extensions from [`data/extensions/level{1..5}.txt`](../crates/drot_dhara_storage/data/extensions/) and ranks survivors to the definition cap.

## Admission rules

1. **Extension floodgate** — at least one listed extension must match levels 1–5.
2. **MIME gate** — canonical IANA/custom MIME (exact or fuzzy repair).
3. **Byte patterns** — non-empty signature required.
4. **Rank + cap** — score by level, pattern strength, MIME confidence; keep top N.

## Seed list policy (modern desktop)

**Keep:** office, PDF, web, common images, mainstream media, archives (`zip`/`7z`/`rar`/`tar`/`gz`), source/config, databases.

**Split-archive helpers (level 4):**

| Token | Reason |
|-------|--------|
| `001`–`040` | 7-Zip / split-ZIP volume suffix (e.g. `archive.7z.001`) |
| `r00`, `r01` | Legacy RAR4 continuation volumes |
| `z01`, `z02` | WinZip split ZIP |

**Remove:** `000` (obsolete first volume), standalone `part` (`.part1.rar` uses extension `rar`), scattered numeric junk, placeholder tokens, niche CAD/EDA/CAM/FEA verticals from levels 3–5.

RAR5 multi-volume (`*.part1.rar`) needs no extra seeds — `rar` in level 1 covers all parts.

## Rebuild

From the host repo: `defs build-trid-xml` then `defs sync-embedded` (via `run-drot`).
