# Citations-rb component inventory

Upstream: `origin/master` (Rust/gtk-rs, GNOME World/Citations). Port: `ruby`
branch, `lib/`. This is the opening survey of the fork: every user-facing
component upstream defines, the `lib/*.rb` file it maps to, its kind, and
whether that file exists on `ruby` today.

Totals: upstream has 1 application class, 1 top-level window, 4 composite
widgets (`EntryList`, `EntryListRow`, `EntryPage`, `CitationPreview`), 3
dialogs (`AddPdfDialog`, `NewEntryDialog`, the About dialog built in code),
1 selection-model helper, 1 standalone shortcuts window with no backing
source, plus the `cratebibtex` and `texer` support crates. **Every one of
these already has a same-named counterpart in `lib/` on `ruby`** — this
port is far along; the open question for the next step is parity (CSS
classes, signals, actions, strings) inside each file, not presence/absence.

## Window and application shell

| Upstream file | Kind | `lib/*.rb` | Exists? | Notes |
|---|---|---|---|---|
| `src/application.rs` | `Adw.Application` subclass — app actions (`app.quit`, `app.new-window`), accels, CLI options (`--new-window`, `--debug`), `open()`/`activate()`/`startup()` | `lib/citations/application.rb`, `lib/main.rb` | Yes | Upstream's `Application` struct + its `setup_gactions`/`setup_accels`; the Ruby port splits bootstrapping into `lib/main.rb` (mirrors `src/main.rs`'s `fn main`) and the `Citations::Application` wrapper used by `exe/citations` |
| `src/window.rs` (`Window`, `+ recent_row()`) | `Adw.ApplicationWindow` subclass — the whole shell: `AdwNavigationSplitView`, recent-files stack/empty-state, search bar, toast overlay, 11 `win.*` actions, drag-and-drop target, `AdwBreakpoint` at 600sp, undo-delete toast, save/open file dialogs, `AdwAlertDialog` unsaved-changes prompts | `lib/window.rb` | Yes | 909 lines; also owns the recent-file `Adw.ActionRow` (`recent_row()` in Rust) — ported separately as its own file |
| `recent_row()` helper in `src/window.rs` | `Adw.ActionRow` + suffix `Gtk.Button` (`edit-delete-symbolic`, `.flat`), one per recent file | `lib/recent_file_row.rb` | Yes | Upstream builds this inline in a closure; the port gives it its own class, consistent with house style |
| `src/config.rs.in` | build-time constants (`APP_ID`, `VERSION`, `PROFILE`, `IS_DEVEL`, …) | `lib/config.rb` | Yes | |
| `src/selection.rs` (`Selection`/`SidebarSelection`) | `Gio::ListModel` + `Gtk::SelectionModel` custom GObject — single-item selection with `hide-selection` | `lib/selection.rb` | Yes | |
| `src/utils.rs` | field-name translation table, Google Scholar/arXiv URL builders, bibliography-path helpers, DOI/eprint URL validation, async HTTP `get`, BibTeX-from-DOI, citation-key generation/sanitizing | `lib/utils.rb` | Yes | |
| `data/org.gnome.World.Citations.gschema.xml.in` | GSettings schema — `window-width`/`window-height`/`is-maximized`/`recent-files`/`custom-book-template`/`custom-article-template`/`template-kind`/`format-kind` | `lib/settings.rb` | Yes | Port can't use real GSettings portably, so `lib/settings.rb` re-implements the same 8 keys over JSON |
| `data/resources/style.css` | application stylesheet | `lib/style.rb` | Yes | |
| (none — built in Rust) | app icon / i18n bootstrap | `lib/i18n.rb` | Yes | mirrors `gettextrs::bindtextdomain`/`textdomain` setup in `src/main.rs` |

## Dialogs

| Upstream file(s) | Kind | `lib/*.rb` | Exists? | Notes |
|---|---|---|---|---|
| `data/resources/ui/add_pdf_dialog.ui` + `src/add_pdf_dialog.rs` | `AdwDialog` — `AdwToastOverlay` > `GtkOverlay` (osd `GtkProgressBar`) > `AdwClamp` > error box, `AdwEntryRow` "Add From URL" (suffix download `GtkButton .flat`), `AdwButtonRow` "Add From File…"; emits `pdf-set` signal | `lib/add_pdf_dialog.rb` | Yes | |
| `data/resources/ui/new_entry_dialog.ui` + `src/new_entry_dialog.rs` | `AdwDialog` — `AdwHeaderBar` with Cancel/Create buttons, `AdwViewStack` (New / From BibTeX / From DOI pages) + `AdwViewSwitcherBar`, `AdwEntryRow`, `AdwComboRow`, `GtkSourceView` (bibtex lang) | `lib/new_entry_dialog.rb` | Yes | |
| `show_about_dialog()` in `src/window.rs` (no `.ui`; built from `data/org.gnome.World.Citations.metainfo.xml.in.in`) | `Adw.AboutDialog` built via `from_appdata` | `lib/about_dialog.rb` | Yes | Ruby bindings lack `from_appdata`, so the port parses the metainfo XML itself (documented gap: no `translate` URL setter found) |
| `data/resources/ui/shortcuts.ui` (standalone, no backing `.rs`) | `AdwShortcutsDialog` — 2 sections ("General", "Bibliography"), 7 `AdwShortcutsItem`s bound to `app.shortcuts`/`app.new-window`/`window.close`/`app.quit`/`win.open`/`win.save`/`win.search` | `lib/shortcuts_dialog.rb` | Yes | Component with no backing Rust source — its own unit per the component-identification rules |
| `ask_opening_new_bib_with_changes()` / `show_save_dialog()` in `src/window.rs` | 2 `AdwAlertDialog` instances (3 responses each: discard/cancel/save) | `lib/window.rb` | Yes | Built inline in `window.rb`, matching upstream's inline construction |

## Sidebar / entry list

| Upstream file(s) | Kind | `lib/*.rb` | Exists? | Notes |
|---|---|---|---|---|
| `data/resources/ui/entry_list.ui` + `src/entry_list.rs` | `AdwBin` subclass — `GtkStack` (empty `AdwStatusPage` / `GtkScrolledWindow` > `GtkListView` with `navigation-sidebar padded-sidebar`), owns `GtkFilterListModel`/`GtkStringFilter` and the `Selection` model, `SignalListItemFactory`, emits `entry-selected` | `lib/entry_list.rb` | Yes | |
| `data/resources/ui/entry_list_row.ui` + `src/entry_list_row.rs` | plain `Gtk.Widget` subclass (`BinLayout`) — author/title/year `GtkLabel`s, `GtkGestureClick`+`GtkGestureLongPress` controllers, `GtkPopoverMenu` (`row_menu`: Open PDF / Copy Key / Delete), 4 `entry-row.*` actions + 2 key bindings (Menu, Shift+F10) | `lib/entry_list_row.rb` | Yes | |

## Entry page / citation preview

| Upstream file(s) | Kind | `lib/*.rb` | Exists? | Notes |
|---|---|---|---|---|
| `data/resources/ui/entry_page.ui` + `src/entry_page.rs` | `AdwBin` subclass — large detail page: `GtkStack` (empty bin / empty `AdwStatusPage` / scrolled content), boxed-list of `AdwComboRow`+`AdwEntryRow`s (type, citation key w/ copy button, author, title, year, DOI w/ open-in-browser button), dynamic "Other Properties" `AdwPreferencesGroup`+`GtkListBox` of per-field `AdwEntryRow`s, Abstract/Notes `AdwPreferencesGroup`s with copy buttons, embeds `CitationPreview`, PDF `GtkStack` (add-PDF `AdwActionRow` vs. `AdwPreferencesGroup` preview with open/delete buttons, `GtkPicture`/error state), 2 `entry-page.*` actions | `lib/entry_page.rb` | Yes | 885 lines — the largest single component; has its own `AddPdfDialog` launch and PDF-render-to-texture pipeline |
| `data/resources/ui/citation_preview.ui` + `src/citation_preview.rs` | `AdwBin` subclass — `GtkRevealer`-wrapped custom-template `AdwPreferencesGroup` (`GtkTextView`), citation `AdwPreferencesGroup` with 2 `GtkDropDown`s (Template/Format enums) and a `GtkSourceView` (syntax highlighted per format, copy button), help `GtkPopover` | `lib/citation_preview.rb` | Yes | Template enum (Mla/Ams/Apa/Chicago/Harvard/Vancouver/Custom) and the 12 citation-string constants live here |

## Menus (from `data/resources/ui/window.ui` and `entry_list_row.ui`)

| Menu | Items | `lib/*.rb` | Exists? |
|---|---|---|---|
| `primary_menu_empty` | Create New Bibliography, Open, Keyboard Shortcuts, About Citations | `lib/window.rb` (`primary_menu_empty`) | Yes |
| `primary_menu` | Save, Open, New Window, Keyboard Shortcuts, About Citations | `lib/window.rb` (`primary_menu`) | Yes |
| `entry_menu` (secondary menu button) | Search in Google Scholar, Search in arXiv, Copy BibTeX Entry, Delete | `lib/window.rb` (`entry_menu`) | Yes |
| `row_menu` (`entry_list_row.ui`) | Open PDF, Copy Key, Delete | `lib/entry_list_row.rb` | Yes |

## Support crates (non-widget, but load-bearing for component behaviour)

| Upstream | Kind | `lib/*.rb` | Exists? |
|---|---|---|---|
| `cratebibtex/src/bib.rs` | the parsed bibliography (`Gio::ListModel` of entries) | `lib/bib.rb` | Yes |
| `cratebibtex/src/entry.rs` | one citation entry (GObject w/ bound properties) | `lib/entry.rb` | Yes |
| `cratebibtex/src/entry_type.rs` | citation-type enum + translated names | `lib/entry_type.rb` | Yes |
| `cratebibtex/src/format.rs` | citation `Format` enum (Plain/Markdown/LaTeX) + author formatting | `lib/citation_format.rb` | Yes |
| `texer/src/lib.rs`, `texer/src/utils.rs` | LaTeX-markup-to-plain-text converter | `lib/texer.rb` | Yes |

## Not yet surveyed in this pass

- **`po/*.po` catalogue** (41 locales + `citations.pot`): not inventoried as
  components; `lib/i18n.rb` wires gettext but no `.po`/`.mo` files were found
  under the `ruby` branch's own tree — translation parity is a separate
  pass (see the `translation-parity` skill).
- **Icons**: upstream ships `document-text-symbolic`,
  `external-link-symbolic`, `image-missing-symbolic` plus the app icon in two
  sizes. The port has its own `data/icons/hicolor/...` tree under the
  `org.rubygtkproject.Citations` id — present, not diffed icon-for-icon here.
- **GResource manifest** (`data/resources/resources.gresource.xml.in`): the
  Ruby port has no `.ui`/GResource step by design (house style builds
  widgets in code), so this has no `lib/` counterpart and is not a gap.
- This pass checked **presence** (every upstream file has a same-named
  `lib/*.rb`), not **parity** (every CSS class/signal/action reproduced
  correctly inside each file). That is the next step, via
  `component-parity`.
