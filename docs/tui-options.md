# TUI options and CLI parity

This document describes how the interactive TUI presents command options differently from the Direct CLI, while still serializing to the same argv the CLI would produce.

## Problem this solves

Production CI and scripts need conservative defaults (Release builds, full verify, all workflow steps). Local interactive use needs faster iteration (Debug builds, fewer steps, host-only native). Duplicating commands or maintaining parallel flag sets would drift from the CLI.

The TUI therefore uses **TUI-specific defaults and presets** that only affect form initialization. [`CommandForm::build_args`](../crates/drot_kernel/src/forms.rs) is shared — whatever the operator selects in the TUI becomes the same `--flag` tokens the Direct CLI accepts.

## Field schema extensions

| Field attribute | CLI | TUI |
| --------------- | --- | --- |
| `default_value` | Initial value / clap default | Ignored for TUI init when `tui_default_value` is set |
| `tui_default_value` | Ignored | Initial value when the form opens |
| `invert_switch` | — | Checked step **includes** the step; unchecked emits the skip flag (e.g. `--skip-verify`) |
| `group` | — | Section heading in the Options tab |
| `tui_nest` | — | Extra indent levels under the group (whole row: control + label) |
| `tui_only` | Not serialized | Preset picker and other UI-only fields |
| `tui_combo_width` | Ignored | Optional inner text slot width; marquee when value overflows |
| `FieldKind::Preset` | — | Workflow preset row; applying a preset sets other fields |

Kernel hooks ([`ProductHooks`](../crates/drot_kernel/src/product.rs)) let the product extension apply presets after TUI init:

- `initialize_tui_form` — run when a command form is first opened or reset
- `apply_tui_preset` — run when the operator picks a different preset

Storage presets live in [`presets.rs`](../crates/drot_dhara_storage/src/registry/presets.rs).

## Presets (storage extension)

### `build.run`

| Preset | Intent |
| ------ | ------ |
| **Local integration** | Debug, host-only native, verify off, docs off — fast local loop |
| **Production parity** | Release, cross-native, verify on, docs on — matches CI shape |
| **Custom** | Operator adjusts individual steps |

Default on open / Reset: **Local integration**.

### `release.run`

| Preset | Intent |
| ------ | ------ |
| **Dry run** | Plan only; no cargo/nuget publish steps |
| **Full release** | Execute cargo and NuGet publish |

Default on open / Reset: **Dry run**.

## Widget kit and embedded focus

Grouped fields render in the Options tab with BIOS-style controls.

### Combo layout and focus

Each combo row is **caption left**, **inner cluster right-aligned**:

```
Label                    [ ⮜ _ {value slot} _ ⮞ ]
```

(`[` and `]` denote the highlighted background — not drawn as glyphs.)

| Region | Click / focus | Visual |
| ------ | ------------- | ------ |
| Label | Enters **inner** focus | Accent fg when row selected; **no chevron prefix** |
| Highlighted cluster, value slot | **Inner** focus | Background highlight; text accent when inner active |
| `⮜`, `⮞` | **Buttons** — cycle prev/next | Distinct button styling; always whole symbols |

**Sizing:**

- Unspecified (`tui_combo_width: None`): text slot = longest option label width; total cluster = slot + 6 chrome columns (`pad` + `⮜` + `pad` + slot + `pad` + `⮞` + `pad`).
- Specified (`tui_combo_width: Some(n)`): text slot = `n`; marquee when the current value is wider.

Pad spaces beside the value slot are fixed and **not** part of marquee scrolling. Short values are **center-aligned** inside the slot.

### Text / path layout and cursor

Each text row is **caption left** (bold `>` when selected), **input cluster right-aligned**:

```
> Label                    [ >{value} ]
```

(`[` and `]` denote the highlighted background — not drawn as glyphs. Pads sit beside the `>` prompt/value.)

| State | Cursor |
| ----- | ------ |
| Idle empty | Static `_` cue inside the highlight |
| Embedded edit | Terminal caret via Ratatui `Frame::set_cursor_position` (usually blinks); `←`/`→`/`Home`/`End` move `InputState.cursor_pos` |

Highlight width grows with the value (minimum text slot, capped by the row). Long values scroll so the caret stays visible.

### Other controls

- **Boolean steps** — checkbox; inverted switches mean “include step” when checked. Nested steps (e.g. `cargo doc` / `dotnet test` under quality) use `tui_nest` so the checkbox and label indent together.
- **Preset** — combo of workflow presets; changing preset applies field values via hooks

When a field has embedded focus, **Tab** commits and exits the field. Footer hints switch to embedded mode (`Tab: exit field`, etc.).

Reset clears embedded focus and re-applies TUI defaults plus the command’s default preset.

## Info tab vs Options tab

- **Options** — human labels, groups, presets, TUI defaults
- **Info** — resolved CLI flag list from `build_args` (what will actually run)

## Debug configuration

The TUI exposes **Debug** and **Release** for native staging. Verify and publish paths remain Release-gated in ops with clear operator messaging when Debug is selected where Release is required.

## Related docs

- [Architecture](architecture.md) — TUI layout and crate DAG
- [TUI operation progress](tui-progress.md) — progress bar lifecycle during runs
