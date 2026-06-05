# Keycode pipeline

How QMK keycodes get from a `qmk_firmware` checkout into the keymap editor's
picker, including the customizable display layer and parameterized
(mod-tap / layer-tap / etc.) keycodes.

## Why this exists

The GUI needs three things the old hand-maintained keycode list could not give:

1. **Stay in sync with QMK.** QMK defines keycodes in
   `qmk_firmware/data/constants/keycodes/*` and versions them. We generate our
   assets straight from a checkout so adding/renaming a keycode upstream is a
   one-command refresh, not a manual edit.
2. **Curate what the user sees.** The raw QMK list is flat and machine-shaped
   (`KC_*` names, internal groups). A separate, hand-authored file controls the
   picker's tabs, ordering, labels, descriptions, and which keys to hide -
   without touching generated data.
3. **Understand composite keycodes.** Mod-tap, layer-tap, layer-mod, one-shot,
   and modified keycodes are 16-bit values with bit-packed parameters. The
   pipeline decodes them for display (split keycaps) and encodes them back when
   the user assigns one.

## Pipeline at a glance

```
qmk_firmware checkout
        │  sync_keycodes.py  (yarn sync:keycodes)        [commit 1]
        ▼
xap-specs/assets/keycodes_<ver>.generated.hjson   (raw catalog, per QMK version)
xap-specs/assets/keycode_display.hjson            (hand-authored curation)
        │  XapConstants::new -> XapKeyCodeCatalog        [commit 2]
        ▼
   KeycodeView  (tabs -> subgroups -> KeyCode[])    KeyCode resolver (get_keycode)
        │  specta -> src/generated/xap.ts (type bridge)
        ▼
   KeycodePicker / BasicKeyboardLayout / KeyLabel   (Vue)   [commits 3-4]
```

The bracketed tags map to the commits in this branch:
1. *Sync QMK keycodes from qmk_firmware*
2. *Add keycode catalog with customizable display and parameterized keycodes*
3. *Add keycode picker components and window-fit utility*
4. *Render keymap with the data-driven picker and parameterized assignment*

## 1. Sync from qmk_firmware

`xap-specs/scripts/sync_keycodes.py` (run via `yarn sync:keycodes`) imports
QMK's own loader (`qmk.keycodes.list_versions` / `load_spec`) from a checkout
pointed to by `--qmk-firmware` or `$QMK_FIRMWARE`. For each keycode spec
version it writes `keycodes_<version>.generated.hjson` into
`xap-specs/assets/`, sorted deterministically (stable keys, sorted maps) so
re-running produces no spurious diffs. Generated files for versions QMK no
longer ships are pruned.

The generated file is plain JSON (kept under an `.hjson` name so the loader can
read both generated and hand-authored assets uniformly):

```jsonc
{
  "version": "0.0.4",
  "keycodes": {
    "0x0004": { "key": "KC_A", "label": "A", "group": "basic" },
    "0x0001": { "key": "KC_TRANSPARENT", "label": "Transparent",
                "group": "internal", "aliases": ["_______", "KC_TRNS"] }
  },
  "ranges": { ... }
}
```

This data is read at runtime from the resource directory; it is not compiled
in, so commit 1 changes no Rust.

## 2. The catalog (`keycode.rs`)

`XapConstants::new` builds an `XapKeyCodeCatalog` (`read_xap_keycode_catalog`)
that loads every `keycodes_*.generated.hjson` plus the single
`keycode_display.hjson`. Per version it precomputes an `XapKeyCodeVersion`:

- `lookup: HashMap<u16, KeyCode>` - code to entry.
- `name_to_code: HashMap<String, u16>` - `KC_*`/alias to code, for the display
  layer's explicit `keys` lists.
- `hidden: HashSet<u16>` - codes the curation file marked hidden.
- `categories` - the raw per-group lists (legacy/raw-fallback view).

`KeyCode` is the single entry type shared end to end (it is what specta exports
to TypeScript):

```rust
pub struct KeyCode {
    pub code: u16,
    pub key: String,                 // KC_A
    pub group: Option<String>,       // basic, layer, mod_tap, ...
    pub label: Option<String>,       // "A"          (NoneAsEmptyString)
    pub top: Option<String>,         // split keycap upper half (hold)
    pub bottom: Option<String>,      // split keycap lower half (tap)
    pub aliases: Vec<String>,
    pub description: Option<String>,
    pub template: Option<KeycodeTemplate>,  // set for parameterized keycodes
}
```

### Version resolution

`XapConstants` exposes `keycode_version` (latest), `keycode_versions` (all), and
a `keycode_view` for the latest. `resolve_version` accepts an exact version name
or any semver string and snaps **down** to the newest catalog version `<=` the
request, so a board reporting an in-between version still gets a sensible view.
Everything defaults to the latest when unspecified.

### Per-key resolution: `get_keycode`

`get_keycode(version, code)` is the resolver the keymap fetch uses for each key:
1. exact `lookup` hit -> return it;
2. else `decode_parameterized` (see below);
3. else `KeyCode::new_custom` (renders as `0xNNNN`).

## 3. Customizable display (`keycode_display.hjson` + `keycode_display.rs`)

`keycode_display.hjson` is hand-authored and decoupled from QMK data. It has its
own `version` and a `target_keycode_version`: **if the active catalog version
does not match `target_keycode_version`, the GUI falls back to a raw view (one
tab per generated group) and none of the curation applies.** This keeps a stale
curation file from silently mislabeling a newer keycode set.

```jsonc
{
  "version": "1",
  "target_keycode_version": "0.0.8",
  "tabs": [
    { "id": "basic", "label": "Basic", "subgroups": [
      { "id": "ansi", "label": "Standard", "render_mode": "ansi",
        "from_groups": ["basic", "modifiers"] },
      { "id": "blank", "label": "Blank", "keys": ["KC_NO", "KC_TRANSPARENT"] }
    ]}
  ],
  "keycodes": {
    "KC_NO": { "label": "", "hidden": true },
    "KC_ENTER": { "description": "Enter / Return" }
  }
}
```

`build_view_for_version` turns this into the wire `KeycodeView`
(`tabs -> KeycodeViewTab -> KeycodeViewSubgroup -> KeyCode[]`). Subgroup
populators, applied in order:

- `keys: [...]` - explicit `KC_*`/alias names, in the given order.
- `from_group` / `from_groups` - every catalog key in those generated groups
  that has not already been claimed by an earlier subgroup (explicit `keys`
  win over a later `from_group` sweep).
- `template` - a `SubgroupTemplate` (e.g. `MO`, `LT`, `MT { mods }`); ships
  un-expanded, and the frontend renders one button per layer / mod combination
  using the connected board's layer count.

`render_mode: "ansi"` flags a subgroup the Vue side draws as a physical
keyboard rather than a button grid. Per-key `keycodes` overrides patch
`label` / `description` / `hidden`; hidden keys are pulled out of normal
subgroups and collected into a synthesized "Hidden" tab.

## 4. Parameterized keycodes (`keycode_decoder.rs` + `keycode_encoder.rs`)

Composite keycodes are 16-bit values with bit-packed parameters. `consts` in
`keycode_encoder.rs` mirrors the QMK range/modifier constants; the
`qmk_header_constants_match_local` test reads the QMK headers at test time and
fails if any drift.

**Decode** (`decode_parameterized`, used by `get_keycode`): classifies a code by
range (QK_MODS, QK_MOD_TAP, QK_LAYER_TAP, QK_LAYER_MOD, the QK_TO..
QK_PERSISTENT_DEF_LAYER layer ops, QK_ONE_SHOT_MOD) and produces a `KeyCode`
with:
- a combined `label` (e.g. `LSFT_T(A)`) for tooltips/palette, and
- `top`/`bottom` split fields for the two-line keycap (`LSFT_T` over `A`),
- the embedded basic keycode resolved recursively through the catalog,
- modifier masks named QMK-style via `mods_name` (`MEH`, `HYPR`, `LCAG`, ...),
- a `template` describing how to rebuild it.

**Encode** (`KeycodeTemplate::encode`): the inverse. `KeycodeTemplate` is a
tagged enum whose `Option` slots model the picker's two-step flow - e.g.
`ModTap { mod_mask, tap_kc: None }` after the user picks the modifier but before
the tap key. `is_complete` gates encoding; `encode` packs the bits back to the
`u16` the firmware expects. Exposed to the frontend as the
`keycode_template_encode` Tauri command.

## 5. Type bridge and frontend

Rust types flow to `src/generated/xap.ts` via `specta`/`tauri-specta` (see the
generated file; regenerate by building the app, then `prettier --write` it
since it is in `.prettierignore`). The exported surface this pipeline adds:
`KeyCode`, `KeycodeView`/`KeycodeViewTab`/`KeycodeViewSubgroup`,
`KeycodeTemplate`/`LayerOp`/`SubgroupTemplate`, the reshaped `XapConstants`
(`keycode_version`, `keycode_versions`, `keycode_view`), and the
`keycodeTemplateEncode` command.

The Vue side consumes the view directly: `KeycodePicker` renders the tabs and
subgroups; `BasicKeyboardLayout` renders `render_mode: "ansi"` subgroups as a
keyboard; `KeyLabel` draws single or split (`top`/`bottom`) keycaps;
`keycodeFamily.ts` maps groups to styling. Template subgroups and individual
template keycodes drive the two-step assignment, which calls
`keycodeTemplateEncode` to produce the value sent to `remap_key`.

### Keymap view sizing

The editor auto-fits its window to the keymap plus the picker
(`fitCurrentWindowToContent` in `windowFit.ts`), clamped so the window never
grows larger than the monitor (the min-size floor is clamped below the screen
cap rather than overriding it). When the screen-capped window still cannot show
at least 50% of the picker's ANSI layout, `KeymapView` caps the keymap display
height and gives it a vertical scrollbar so the ANSI layout is brought back into
view. The cap is measured against the keymap's natural height (so it never feeds
back into window sizing) and recomputed on viewport and picker-tab size changes.

## Maintenance

- **Refresh keycodes:** `yarn sync:keycodes --qmk-firmware <path>` (or set
  `$QMK_FIRMWARE`), then commit the regenerated assets.
- **Adopt a new keycode version in the picker:** bump
  `target_keycode_version` in `keycode_display.hjson` and reconcile the tabs;
  until then newer catalogs render via the raw fallback view.
- **Guardrail:** the `qmk_header_constants_match_local` test (run `cargo test`
  in `xap-specs`) catches QMK constant drift in the encoder/decoder.

## Key files

| File | Role |
|------|------|
| `xap-specs/scripts/sync_keycodes.py` | Generate versioned catalogs from qmk_firmware |
| `xap-specs/assets/keycodes_*.generated.hjson` | Generated raw keycode catalogs |
| `xap-specs/assets/keycode_display.hjson` | Hand-authored tabs/labels/overrides |
| `xap-specs/src/constants/keycode.rs` | `XapKeyCodeCatalog`, `KeyCode`, version resolution, `get_keycode` |
| `xap-specs/src/constants/keycode_display.rs` | Display schema and `KeycodeView` builder |
| `xap-specs/src/constants/keycode_decoder.rs` | Decode parameterized keycodes |
| `xap-specs/src/constants/keycode_encoder.rs` | `KeycodeTemplate`, encode, QMK constants |
| `src-tauri/src/rpc/commands.rs` | `keycode_template_encode` command |
| `src/components/{KeycodePicker,BasicKeyboardLayout,KeyLabel}.vue` | Picker UI |
| `src/pages/KeymapView.vue` | Keymap editor; renders the picker, caps/scrolls the keymap when space is tight |
| `src/utils/windowFit.ts` | Fit the window to content, clamped to the monitor |
