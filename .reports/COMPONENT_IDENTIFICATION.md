# Component identification — Citations (audit pass, 2026-11-07, round 2)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `e4ebd9f` |
| Runtime gate this round | `drive_ok=true`, 74/74 checks, 0 failed, 15 components claimed |

This is a second pass. The previous round's three misses — the dropped
gschema file, the undriven Add PDF dialog, and the `EntryList#filter`
GObject-CRITICAL — were all addressed in commit `e4ebd9f` and are
re-verified below as fixed, not re-opened. This pass's job was to find what
that commit did *not* catch.

## Prior misses, re-verified fixed

| Miss (round 1) | Status | Evidence |
|---|---|---|
| No `gschema.xml.in` file, 7 summary strings unported | **fixed** | `data/org.rubygtkproject.Citations.gschema.xml.in` exists, all 8 keys and all 7 `<summary>` strings byte-identical to upstream's (including upstream's own duplicate "Default template for book citation" on both template keys, kept as-is) |
| `lib/add_pdf_dialog.rb` had zero runtime coverage | **fixed** | `test/drive_main_window.rb` checks 33–38: opens the dialog from the empty PDF row, confirms it is titled "Add PDF" and empty, types a bad URL, confirms the download is rejected without a network call and the entry keeps the error style, cancels, confirms the entry is still PDF-less |
| `GLib-GObject-CRITICAL` on every `EntryList#filter` build | **fixed** | `lib/entry_list.rb` now builds `Gtk::StringFilter.new` with a nil expression and assigns the real one through the setter; `test/gtk_driver.rb` now watches stderr for the whole drive run and fails on any CRITICAL line — reproduced a clean `DRIVE OK` with 74/74 this pass, no CRITICAL emitted |
| RSpec banner advertised a suite that cannot load, with zero spec files | **fixed** | `spec/` is gone entirely; the dev-shell banner now names `test/test_models.rb` and `test/test_texer.rb` (the real 22 ported upstream tests) and the `xvfb-run` requirement; see `TEST_PARITY.md` |

## New miss — the devel-profile distinction is silently absent

Upstream's `config.rs.in` carries `IS_DEVEL: bool`, set by `meson.build`
from the build profile, and used in exactly two places in `src/window.rs`:

1. **`constructed()` (line ~234)**: when `IS_DEVEL`, adds the `devel` CSS
   class to the window (the red diagonal "Devel" ribbon GNOME apps show in
   nightly/debug builds) and sets both empty-state status pages'
   `icon-name` to the app id's full-colour icon.
2. **`show_about_dialog()` (line ~677)**: when `IS_DEVEL`, sets the About
   dialog's version string explicitly to the full `VERSION` (e.g. with a
   git commit suffix); otherwise the version shown is whatever
   `Adw::AboutDialog::from_appdata` derives from the metainfo release list.

The port's `lib/config.rb` carries the same constant —
`Config::IS_DEVEL = false` — but it is **read nowhere**. `grep -rn
'IS_DEVEL' lib/ exe/` returns only its own definition.

- `lib/window.rb` never adds the `devel` CSS class under any condition —
  confirmed against the full component scan: `devel` is the one CSS class
  present in upstream's scan output and absent from the port's.
- `lib/window.rb`'s `no_recent_files_status_page` and
  `recent_files_status_page` both set `page.icon_name = Config::APP_ID`
  **unconditionally** (lines 225, 250) — upstream only does this when
  `IS_DEVEL`; in a stable build those status pages fall back to GTK's
  default icon rendering, not the full-colour app icon. The port always
  shows the devel-only icon, in every build.
- `lib/about_dialog.rb#initialize` sets `about.version = @version`
  unconditionally (line 58) with no `IS_DEVEL` gate at all.

This is a real, three-site behavioural gap, present in a file that already
defines the exact constant upstream gates on and never consults it. Nothing
in `lib/window.rb`, `lib/about_dialog.rb`, `lib/config.rb`'s comment, or any
`.reports/` file mentions this tradeoff — it was simply never ported. See
`COMPONENT_PARITY.md` for the full before/after table and
`ACCOUNTABILITY_ENSURANCE.md` for why "never mentioned" is worse than an
excuse.

## Held up this pass

Full widget/CSS/signal/action scan re-run both sides this pass
(`scripts/component-scan.sh`, symbol-table assisted). Outside the `devel`
CSS class, every CSS class upstream's scan reports has a port counterpart
and vice versa; all 8 gschema keys, all `win.*`/`app.*`/`entry-page.*`/
`entry-row.*` actions, and all 6 accelerators (`app.quit`, `app.new-window`,
`win.open('')`, `win.save`, `win.search`, `window.close`) match exactly.
The shortcuts dialog (`shortcuts.ui` ↔ `lib/shortcuts_dialog.rb`) matches
item-for-item, including the honestly-documented `app.shortcuts`
self-install workaround for the missing libadwaita binding. The six
built-in citation templates plus Custom, the 1:1 Rust-crate-to-`lib/`-file
mapping (`cratebibtex` → `bib.rb`/`entry.rb`/`entry_type.rb`/
`citation_format.rb`, `texer` crate → `texer.rb`), and the `Gtk.SourceView`
→ plain `Gtk.TextView` syntax-highlighting downgrade in
`new_entry_dialog.rb`/`citation_preview.rb` (openly documented in both
files' class comments, not hidden) were all re-checked and held up.

## Verdict

Not clean. Three prior misses confirmed fixed. One new, unexcused,
undocumented gap found: the devel-profile distinction (CSS class, two
status-page icons, About dialog version string) that upstream gates on
`IS_DEVEL` and the port's own `Config::IS_DEVEL` constant is never read.
