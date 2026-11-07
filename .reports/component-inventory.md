# Component inventory — Citations

Survey of upstream (`origin/master`) against the `ruby` port's `lib/`, done
with the `component-identification` skill. One row per upstream unit (a
source file, usually paired with its `.ui`), the port file it maps to, its
kind, and whether that port file already exists.

Headline: the `ruby` branch is essentially complete at the file-mapping
level — every upstream `src/*.rs` and `data/resources/ui/*.ui` file already
has a corresponding `lib/*.rb`, each carrying the "ported from" comment the
house style requires. This inventory does **not** verify widget-for-widget
parity inside each file (counts, CSS classes, signals, dynamic state) — that
is the `component-parity` skill's job and should be the next step, run file
by file in the order below.

## Units, in the order a user meets them

| # | Upstream file(s) | → `lib/` path | Kind | Exists? |
|---|---|---|---|---|
| 1 | `src/application.rs` | `lib/citations/application.rb` (aliases `App` from `lib/main.rb`) | application class (actions, accelerators, CLI options, `About`/`Preferences`/`shortcuts` entry points) | Yes |
| 2 | `src/main.rs` | `lib/main.rb` | app bootstrap / `App` class | Yes |
| 3 | `src/config.rs.in` | `lib/config.rb` | build-time constants | Yes |
| 4 | `src/window.rs` + `data/resources/ui/window.ui` | `lib/window.rb` | main `Adw.ApplicationWindow` (header bar, split view, toasts, open/save/search actions) | Yes |
| 5 | `data/resources/ui/shortcuts.ui` (no backing `.rs`; standalone template) | `lib/shortcuts_dialog.rb` | keyboard-shortcuts dialog (`Adw.ShortcutsDialog`, General + Bibliography sections) | Yes |
| 6 | (no upstream `about.rs`; `Adw.AboutDialog` built inline from metainfo/config) | `lib/about_dialog.rb` | About dialog | Yes |
| 7 | `src/entry_list.rs` + `data/resources/ui/entry_list.ui` | `lib/entry_list.rb` | sidebar list of bibliography entries (`Gtk.ListView`/filter/search) | Yes |
| 8 | `src/entry_list_row.rs` + `data/resources/ui/entry_list_row.ui` | `lib/entry_list_row.rb` | one row in the entry list | Yes |
| 9 | `src/entry_page.rs` + `data/resources/ui/entry_page.ui` | `lib/entry_page.rb` | entry detail/edit page (fields, citation preview trigger) | Yes |
| 10 | `src/add_pdf_dialog.rs` + `data/resources/ui/add_pdf_dialog.ui` | `lib/add_pdf_dialog.rb` | "add from PDF" dialog | Yes |
| 11 | `src/new_entry_dialog.rs` + `data/resources/ui/new_entry_dialog.ui` | `lib/new_entry_dialog.rb` | "new entry" type-picker dialog | Yes |
| 12 | `src/citation_preview.rs` + `data/resources/ui/citation_preview.ui` | `lib/citation_preview.rb` | rendered-citation preview dialog/pane | Yes |
| 13 | (recent-files row; part of `window.rs`'s open flow, no separate upstream file) | `lib/recent_file_row.rb` | row for a recently opened file | Yes |
| 14 | `src/selection.rs` | `lib/selection.rb` | single-selection model for the sidebar | Yes |
| 15 | `src/utils.rs` | `lib/utils.rb` | shared helpers (incl. HTTP fetch) | Yes |
| 16 | `cratebibtex/src/bib.rs` | `lib/bib.rb` | BibTeX file parser/writer | Yes |
| 17 | `cratebibtex/src/entry.rs` | `lib/entry.rb` | one bibliography entry (model) | Yes |
| 18 | `cratebibtex/src/entry_type.rs` | `lib/entry_type.rb` | entry-type enum (book, article, …) | Yes |
| 19 | `cratebibtex/src/format.rs` | `lib/citation_format.rb` | citation output format enum | Yes |
| 20 | `texer/src/lib.rs` + `texer/src/utils.rs` | `lib/texer.rb` | LaTeX-markup-to-plain-text converter | Yes |
| 21 | `data/resources/style.css` | `lib/style.rb` | application stylesheet | Yes |
| 22 | `data/org.gnome.World.Citations.gschema.xml.in` | `lib/settings.rb` | GSettings-backed settings (all 8 keys present: `window-width`, `window-height`, `is-maximized`, `recent-files`, `custom-book-template`, `custom-article-template`, `template-kind`, `format-kind`) | Yes |
| 23 | `po/*.po` (gettext catalogue, 40 locales) | `lib/i18n.rb` | i18n binding (`I18n::Translatable`, `p_`/gettext) | Yes (binding present; actual `.po`/`.pot` catalogue is a separate `translation-parity` concern, not surveyed here) |
| — | `data/org.gnome.World.Citations.metainfo.xml.in.in` | consumed by `lib/about_dialog.rb` / `lib/config.rb` | app metadata (name, summary, release notes, URLs) | Yes, at the level checked (About dialog exists); release-note freshness not verified |
| — | `data/org.gnome.World.Citations.desktop.in.in`, `.service.in` | — | desktop launcher / D-Bus service files | N/A to `lib/` — these are packaging, not widget code; no `.rb` counterpart expected |
| — | `data/icons/*` | — | app icons | N/A to `lib/` |
| — | `data/resources/icons/*.svg` | consumed inline by `lib/entry_page.rb` et al. | inline UI icons (document-text, external-link, image-missing) | Not independently verified this pass — check usage sites when auditing `entry_page`/`citation_preview` for parity |

## Gaps found this pass

None at the file-mapping level: every upstream source/UI unit has a
same-named or sensibly-named `lib/*.rb` counterpart, and every file opens
with the house-style header (`# frozen_string_literal: true`, `require
'adwaita'`/`'gtk4'`, `require_relative` siblings) and names the upstream file
it was ported from in a leading comment.

## What this pass did not check

This was a `component-identification` survey (file-level mapping), not a
`component-parity` audit. Not yet verified, and recommended as the next
pass, file by file in the order above:

- Widget **counts**, CSS classes and signal connections inside each `lib/*.rb`
  against the matching upstream `.rs`/`.ui` pair (e.g. does
  `lib/entry_page.rb` reproduce every field row, dynamic CSS class and
  `notify::*` handler in `entry_page.rs` + `entry_page.ui`?).
- The `app.*`/`win.*` **action** surface — action names are frequently built
  from loops or constants on both sides and need reading, not grepping.
- **GResource manifest** (`data/resources/resources.gresource.xml.in`) against
  what `lib/style.rb` and friends actually load at runtime.
- **`po/` catalogue** completeness — covered by the `translation-parity`
  skill, not this one.
- The **metainfo release-notes** list, to see if the port's About dialog
  surfaces the same version history.

## Scan artifacts

Produced with `.claude/skills/component-identification/scripts/{gir-symbols.sh,component-scan.sh}`:
upstream tree scanned 329 rows / port `lib/` scanned 242 rows (raw grep-based
counts, not yet reconciled per file — see "What this pass did not check").
