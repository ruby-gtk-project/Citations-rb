# Citations — upstream component inventory

Survey of every user-facing component on `origin/master` (the Rust/GTK4
implementation of GNOME Citations) against what `lib/` on the `ruby` branch
holds today.

Read with the `component-identification` skill: a component is a widget plus
its CSS classes, its signals, the actions it installs or targets, and the file
that builds it. Upstream splits most components across two files — a
`src/*.rs` subclass and a `data/resources/ui/*.ui` GtkBuilder template — so the
rows of both are unioned before comparing.

## State of the port

`lib/` on `ruby` now holds a ported file for every component below. The
survey that opened this file found a bare scaffold; since then the models, the
helpers, the window, the sidebar, the editor, the preview, the two dialogs,
the shortcuts dialog, the translation catalogues and the desktop entry have
all been written, so the column that mattered on the first pass — "does this
exist yet" — is now **present** everywhere, and what is left to check is
parity inside each file rather than its absence.

| File | Lines | Unit it carries |
|---|---:|---|
| `lib/main.rb` | 235 | the application object, actions and accelerators |
| `lib/window.rb` | 909 | the main window, front page, menus, split view |
| `lib/recent_file_row.rb` | 75 | one row of the Recent Files list |
| `lib/about_dialog.rb` | 140 | the About Citations dialog |
| `lib/entry_list.rb` | 200 | the sidebar list |
| `lib/entry_list_row.rb` | 330 | one citation row |
| `lib/entry_page.rb` | 885 | the citation editor |
| `lib/citation_preview.rb` | 409 | the formatted citation |
| `lib/new_entry_dialog.rb` | 304 | Add new Entry |
| `lib/add_pdf_dialog.rb` | 330 | Add PDF |
| `lib/shortcuts_dialog.rb` | 77 | the shortcuts dialog |
| `lib/selection.rb` | 136 | the sidebar selection model |
| `lib/utils.rb` | 317 | helpers behind the UI |
| `lib/bib.rb` | 332 | the bibliography list model (`cratebibtex`) |
| `lib/entry.rb` | 319 | one citation (`cratebibtex`) |
| `lib/entry_type.rb` | 186 | the 16 entry types and their extra fields |
| `lib/citation_format.rb` | 137 | Plain / Markdown / LaTeX |
| `lib/texer.rb` | 375 | LaTeX → plain text (`texer`) |
| `lib/settings.rb` | 123 | the GSettings keys |
| `lib/style.rb` | 57 | the stylesheet, inline |
| `lib/config.rb` | 24 | the build-time constants |
| `lib/i18n.rb` | 49 | the gettext domain |
| `lib/citations.rb`, `lib/citations/application.rb`, `lib/citations/rb/version.rb` | 3 + 10 + — | the gem-facing shell `exe/citations` requires |

Target paths follow the pilot's shape: the ported components are flat
`lib/<thing>.rb` files, the app class is `lib/main.rb`, and
`lib/citations/application.rb` stays as the gem-facing shell.

## Totals

Both columns from `component-scan.sh`; upstream's widget column counts the
`.ui` files, where upstream's trees live, and the port's counts `lib/`.

| | upstream | `ruby` branch |
|---|---:|---:|
| GtkBuilder templates / declarative roots | 9 (8 `template`, 1 named `AdwShortcutsDialog`) | n/a — widgets are built in code |
| `Gtk*`/`Adw*` widget instantiations | 155 (in `.ui`) | 152 (45 distinct types) |
| distinct CSS classes | 21 (in `.ui`) | 21 — the same 21 |
| distinct signals | — | 29 |
| event controllers | — | 5 |
| CSS rules in `data/resources/style.css` | 5 selectors, 2 app-defined classes (`padded-sidebar`, `newentrydialog`) | carried inline in `lib/style.rb` |
| `GAction`s installed | 15 (`app.` 2, `win.` 11, `entry-page.` 2) + 4 `entry-row.` | 24 distinct action names across 34 sites |
| accelerators | 6 | 6, in `lib/main.rb` |
| GSettings keys | 8 | 8, in `lib/settings.rb` |
| `po/` catalogues | 42 locales, 21 POTFILES entries | 42 of 42, under `po/` and `po/<locale>/` |

Widget counts per upstream `.ui` file: `entry_page.ui` 46, `window.ui` 31,
`citation_preview.ui` 23, `new_entry_dialog.ui` 17, `add_pdf_dialog.ui` 16,
`shortcuts.ui` 10, `entry_list_row.ui` 8, `entry_list.ui` 4.

The 21 CSS classes, present on both sides: `boxed-list`,
`boxed-list-separate`, `card`, `compact`, `destructive-action`, `dimmed`,
`error`, `flat`, `heading`, `icon-dropshadow`, `inline`, `navigation-sidebar`,
`newentrydialog`, `numeric`, `osd`, `padded-sidebar`, `pill`, `subtitle`,
`suggested-action`, `title`, `title-1`. Two more are applied from source only:
`devel` (on the window, devel profile) and `error` (added/removed on
`citation_key_entry` and `pdf_url_entry_row` on validation failure).

---

## src/application.rs + src/main.rs — the application object

→ `lib/main.rb` · **present**

Not a widget class, but it owns the actions, the accelerators and the
multi-window behaviour.

| Item | Detail |
|---|---|
| class | `adw::Application`, app id `org.gnome.World.Citations`, resource base path `/org/gnome/World/Citations/`, flags `HANDLES_OPEN` |
| CLI options | `--new-window`/`-w` ("Open a new window"), `--debug`/`-d` ("Enable debug messages") |
| actions | `app.quit` (closes the active window first so window state is saved, then quits), `app.new-window` (new `Window` in its own `Gtk::WindowGroup`) |
| accelerators | `app.quit`→`<Control>q`, `app.new-window`→`<Control>n`, `win.open('')`→`<Control>o`, `win.save`→`<Control>s`, `win.search`→`<Control>f`, `window.close`→`<Control>w` |
| startup | `gsv::init()`, `Gtk::Window.set_default_icon_name(APP_ID)` |
| open | one `Window` per file on the command line, each presented then `window.open(file)` |
| i18n | `setlocale`, `bindtextdomain`, `textdomain`, `glib::set_application_name(gettext("Citations"))` |

`app.shortcuts` is referenced by both primary menus and by `shortcuts.ui`, but
is **not** installed in `setup_gactions` — it comes from libadwaita's automatic
shortcuts-dialog action (the gresource aliases `ui/shortcuts.ui` to
`shortcuts-dialog.ui`, which Adwaita picks up). The port has to install it
itself.

## src/window.rs + data/resources/ui/window.ui — the main window

→ `lib/window.rb` · **present**

The largest unit. An `AdwApplicationWindow` whose content is an
`AdwToastOverlay` wrapping a `GtkStack` that switches between the recent-files
front page and the split-view workspace.

| Widget | × | CSS | Signals / actions | Notes |
|---|---:|---|---|---|
| `Adw.ApplicationWindow` | 1 | `devel` (devel profile only) | `close_request` override | title "Citations"; saves/restores `window-width`, `window-height`, `is-maximized` |
| `Adw.Breakpoint` | 1 | — | — | `max-width: 600sp` → `nav_split_view.collapsed = True` |
| `Adw.ToastOverlay` | 1 | — | — | `toast_overlay`, target of `send_toast` |
| `Gtk.Stack` | 2 | — | — | `main_stack` (front page ↔ split view), `recent_files_stack` (empty ↔ recent) |
| `Adw.ToolbarView` | 4 | — | — | one per front-page state, one per split-view pane |
| `Adw.HeaderBar` | 4 | — | — | two front-page bars, sidebar bar (`show-title=False`), content bar (`show-title=False`) |
| `Gtk.MenuButton` | 4 | — | — | 2 × `primary_menu_empty`, `appmenu_button`→`primary_menu`, `secondary_menu_button`→`entry_menu` (`view-more-symbolic`, starts invisible) |
| `Adw.NavigationSplitView` | 1 | — | `notify::collapsed`→`on_collapsed_notify` | min 300 / max 360 sidebar, fraction 0.35, `show-content=False` |
| `Adw.NavigationPage` | 2 | — | content page `hidden`→`on_content_hidden` | titles "Sidebar", "Content" |
| `Gtk.Button` (new entry) | 1 | — | `win.new_entry` | `list-add-symbolic`, starts insensitive, tooltip "Add New Citation" |
| `Gtk.ToggleButton` | 1 | — | bidirectional bind to `search_bar.search-mode-enabled` | `search_button`, `system-search-symbolic` |
| `Gtk.SearchBar` | 1 | — | `notify::search-mode-enabled`→`on_search_mode_enabled_notify` | clears the entry when search closes; `set_key_capture_widget(window)` once a file is open |
| `Gtk.SearchEntry` | 1 | — | `changed`→`on_changed`, `previous-match`, `next-match` | placeholder "Search…"; filters `entry_list`, arrows navigate it |
| `EntryList` | 1 | — | `entry-selected`→`on_entry_selected` | see its own section |
| `EntryPage` | 1 | — | — | see its own section |
| `Gtk.DropTarget` (controller) | 1 | — | `drop` | name `file-drop-target`, COPY\|MOVE, `Gdk::FileList`; opens the first dropped file, asking about unsaved changes first |

### Front page — empty state

`empty_state_stack_page` / `no_recent_files_status_page`:

| Widget | × | CSS | Signals / actions | Notes |
|---|---:|---|---|---|
| `Adw.StatusPage` | 1 | `icon-dropshadow` | — | icon `org.gnome.World.Citations`, title "Open a BibTeX File", description "Or drag-and-drop a file" |
| `Gtk.Button` | 1 | `pill`, `suggested-action` | `win.open` with target `''` | label "_Open", underline, centered, can-shrink |

### Front page — recent files

`recent_files_stack_page` / `recent_files_status_page`:

| Widget | × | CSS | Signals / actions | Notes |
|---|---:|---|---|---|
| `Adw.StatusPage` | 1 | `icon-dropshadow` | — | title "Manage your Citations" |
| `Adw.Clamp` | 1 | — | — | maximum-size 450 |
| `Adw.PreferencesGroup` | 1 | — | — | title "Recent Files"; header-suffix is an `Adw.ButtonContent` button (`document-open-symbolic`, "_Open", `flat`) firing `win.open::''` |
| `Gtk.ListBox` | 1 | `boxed-list` | — | `recent_files_list`, selection-mode none, `bind_model` over a `Gio::ListStore` of `Gio::File` |

### Recent file row (repeater)

`recent_row(file)` in src/window.rs, one per existing recent path, newest first
→ `lib/recent_file_row.rb` · **present**

| Widget | × | CSS | Signals / actions | Notes |
|---|---:|---|---|---|
| `Adw.ActionRow` | 1 per file | — | `win.open::<path>` | title = basename, subtitle = path unless it starts with `/run/user`, activatable |
| `Gtk.Button` | 1 per row | `flat` | `clicked` → `utils::remove_recent_file` | `edit-delete-symbolic`, tooltip "Remove From Recent Files" |

Non-existent paths are filtered out; an empty list switches
`recent_files_stack` to the empty state. The list is rebuilt whenever the
`recent-files` setting changes.

### Window menus

`primary_menu_empty` (front page), `primary_menu` (workspace), `entry_menu`
(content header) → `lib/window.rb` · **present**

| Menu | Items |
|---|---|
| `primary_menu_empty` | "_Create New Bibliography" `win.new_biblio`; "_Open" `win.open` target `''` ‖ "_Keyboard Shortcuts" `app.shortcuts`; "_About Citations" `win.about` |
| `primary_menu` | "_Save" `win.save`; "_Open" `win.open` target `''`; "New _Window" `app.new-window` ‖ "_Keyboard Shortcuts" `app.shortcuts`; "_About Citations" `win.about` |
| `entry_menu` | "_Search in Google Scholar" `win.search-google-scholar`; "_Search in arXiv" `win.search-arxiv` ‖ "_Copy BibTeX Entry" `win.copy_entry_bibtex` ‖ "_Delete" `win.delete_entry` |

### Window actions

| Action | Parameter | Behaviour |
|---|---|---|
| `win.open` | `s` | empty string → `Gtk::FileDialog` filtered to "BibTeX Files" (`text/x-bibtex`, `*.bib`); otherwise opens that path. Asks about unsaved changes first |
| `win.new_biblio` | — | `Gtk::FileDialog` save, title "Create New Bibliography", accept label "_Create", initial name `bibliography.bib`; creates then opens it. Disabled once a file is open |
| `win.new_entry` | — | runs `NewEntryDialog`; toast "Could not add entry" on failure. Starts disabled |
| `win.save` | — | writes the bib to its file, toast "Bibliography saved". Starts disabled |
| `win.delete_entry` | — | deletes the selected entry. Starts disabled |
| `win.copy_entry_bibtex` | — | serialises the entry to the clipboard, toast "Copied" |
| `win.search-google-scholar` | — | `Gtk::UriLauncher` on a scholar.google.com advanced query built from the texer'd author and title |
| `win.search-arxiv` | — | `Gtk::UriLauncher` on an arxiv.org advanced-search URL |
| `win.undo_delete` | — | re-inserts the last deleted entry at its old position |
| `win.about` | — | the about dialog |
| `win.search` | — | toggles the search bar; collapses the content pane first when the split view is collapsed |

### Window dialogs, toasts and states

| Component | Kind | Detail |
|---|---|---|
| About dialog | `Adw.AboutDialog` | `from_appdata` on the bundled metainfo; translator credits, developer "Maximiliano Sandoval", designer "Tobias Bernard"; shows the full version on devel |
| "Unsaved Changes" / opening | `Adw.AlertDialog` | body "Do you want to write all changes to the bibliography before opening the new one?"; responses `discard` "_Open Without Saving" (destructive), `cancel` "_Don't Open", `save` "_Save and Open" (default) |
| "Unsaved Changes" / closing | `Adw.AlertDialog` | body "Do you want to write all changes to the bibliography?"; responses `discard` "_Quit Without Saving" (destructive), `cancel` "_Don't Quit", `save` "_Save and Quit" (default); raised from `close_request` |
| Delete toast | `Adw.Toast` | "Citation Deleted" + button "Undo" → `win.undo_delete`; a second delete dismisses the pending one |
| Other toasts | `Adw.Toast` | "Bibliography saved", "Copied", "Could not add entry", "An entry with the given citation key already exists", "Citation Key already exists", "PDF added" |

## src/window.rs (`show_about_dialog`) — the About Citations dialog

→ `lib/about_dialog.rb` · **present**

Not a template of its own upstream: `Window::show_about_dialog` builds an
`Adw.AboutDialog` with `from_appdata` on the bundled metainfo and then adds the
credits. The port extracts it to its own file because the Ruby bindings do not
expose `from_appdata`, so the metainfo (`data/org.rubygtkproject.Citations.metainfo.xml`)
is parsed and the same fields are set by hand.

| Item | Detail |
|---|---|
| widget | `Adw.AboutDialog` ×1 |
| fields | name, summary, developer ("Maximiliano Sandoval"), designer ("Tobias Bernard"), translator credits, home page, issue tracker, translation page, newest release notes, GPL-3.0-or-later |
| action | presented by `win.about` |
| devel | shows the full version on a devel build |

## data/resources/ui/entry_list.ui + src/entry_list.rs — the sidebar list

→ `lib/entry_list.rb` · **present**

An `AdwBin` with a stack between an empty state and the scrolled list.

| Widget | × | CSS | Signals / actions | Notes |
|---|---:|---|---|---|
| `Gtk.Stack` | 1 | — | — | empty state ↔ scrolled window, driven by `items-changed` |
| `Adw.StatusPage` | 1 | `compact` | — | `document-text-symbolic`, title "No Citations", description "Create a new citation" |
| `Gtk.ScrolledWindow` | 1 | — | — | `hscrollbar-policy=never` |
| `Gtk.ListView` | 1 | `navigation-sidebar`, `padded-sidebar` | `activate`→`on_list_activate` | `single-click-activate`; `SignalListItemFactory` setting up an `EntryListRow` per item |

Behaviour: own signal `entry-selected(Entry?)`, emitted from
`notify::selected-item` on the custom `Selection`; a
`Gtk::FilterListModel` + `Gtk::StringFilter` (substring, case-insensitive) over
a closure expression of `"{author} {title} {year} {citation_key}"`; methods
`populate`, `filter`, `set_selected` (with `scroll_to`), `hide_selection`,
`show_selection`, `unselect`, `navigate_next`, `navigate_previous`.
`padded-sidebar` is an app-defined class from `style.css`.

## data/resources/ui/entry_list_row.ui + src/entry_list_row.rs — one citation row

→ `lib/entry_list_row.rb` · **present**

A bare `Gtk.Widget` subclass with a `BinLayout`.

| Widget | × | CSS | Signals / actions | Notes |
|---|---:|---|---|---|
| `Gtk.Box` | 2 | — | — | outer vertical, spacing 6, margins 6/6/12/12; inner horizontal, spacing 6 |
| `Gtk.Label` (author) | 1 | `title` | — | hexpand, xalign 0, ellipsize end, 2 lines, wrap word-char; bound to `author` through `pretty_author` (texer'd, "Author not set" when empty) |
| `Gtk.Label` (year) | 1 | `dimmed`, `numeric` | — | yalign 0, bound to `year` |
| `Gtk.Label` (title) | 1 | `subtitle` | — | ellipsize end, 2 lines, wrap word-char, xalign 0, hexpand; bound through `pretty_title` ("Title not set" when empty) |
| `Gtk.PopoverMenu` | 1 | — | — | `row_menu`, no arrow, halign start, position bottom |
| `Gtk.GestureClick` (controller) | 1 | — | `pressed`→`menu_gesture_pressed` | button 0, exclusive; only claims events that `triggers_context_menu` |
| `Gtk.GestureLongPress` (controller) | 1 | — | `pressed`→`menu_long_gesture_pressed` | delay-factor 2, touch-only, exclusive |

`row_menu`: "_Open PDF" `entry-row.open-pdf`; "_Copy Key" `entry-row.copy-key`
‖ "_Delete" `entry-row.delete`.

Actions and keys: `entry-row.open-pdf` (enabled only when the PDF exists),
`entry-row.delete`, `entry-row.copy-key` (clipboard + toast "Copied"),
`entry-row.popup`, bound to `Menu` and `<Shift>F10`. The row sets the
`has-popup` accessible property and points the popover at the click coordinates
(RTL-aware fallback).

## data/resources/ui/entry_page.ui + src/entry_page.rs — the citation editor

→ `lib/entry_page.rb` · **present**

The widest single unit: 46 widgets in the template. An `AdwBin` with a
three-way stack — an empty `AdwBin`, a "No Citation Selected" status page, and
the editor itself.

| Widget | × | CSS | Signals / actions | Notes |
|---|---:|---|---|---|
| `Gtk.Stack` | 3 | — | — | `stack` (3 states), `pdf_stack` (add ↔ preview, crossfade), `preview_error_stack` (picture ↔ error, crossfade) |
| `Adw.Bin` | 1 | — | — | `empty_bin` — blank, so both split-view panes never show a status page at once |
| `Adw.StatusPage` | 1 | — | — | `document-text-symbolic`, title "No Citation Selected" |
| `Gtk.ScrolledWindow` | 4 | 3 × `card` | — | page scroller (`hscrollbar-policy=never`) plus abstract/notes/PDF scrollers (height-request 180) |
| `Adw.Clamp` | 1 | — | — | margins 12/12/36/36 |
| `Gtk.Label` (type) | 1 | `title-1` | — | `entry_type_label`, bound to `entry-type` via `to_translatable_string` |
| `Gtk.Label` (key) | 1 | `dimmed` | — | `citation_label`, bound to `citation-key` |
| `Gtk.ListBox` | 3 | `boxed-list` ×3 | — | main field list, `extras_list_box`, `pdf_add_list_box` |
| `Adw.ComboRow` | 1 | — | `notify::selected`→`on_entry_type_selected_notify` | "Citation _Type"; `GtkEnumList` over `EntryType` with the `drop_down_entry_type_name` closure ("Other…" for `Other`) |
| `Adw.EntryRow` | 6 | `numeric` (year) | see below | `other_entry_type_row` ("_Other Citation Type", hidden unless type is Other, `changed`→`on_entry_type_entry_row_changed`), `citation_key_entry` ("_Citation Key", show-apply-button, `apply`→`on_citation_key_apply`), `author_entry`, `title_entry`, `year_entry`, `doi_entry` ("_DOI") |
| `Gtk.Button` (copy key) | 1 | `flat` | `clicked`→`on_copy_key_clicked` | `edit-copy-symbolic`, tooltip "Copy" |
| `Gtk.Button` (DOI) | 1 | `flat` | `clicked`→`on_doi_clicked` | `doi_button`, `external-link-symbolic`, tooltip "Open in Browser", sensitive only when the DOI parses |
| `Adw.PreferencesGroup` | 4 | — | — | `extras_box` "Other Properties", `abstract_preferences_group` "Abstract", `notes_preferences_group` "Notes", `pdf_preview` "PDF" — all start invisible |
| `Gtk.TextView` | 2 | `inline` | — | `abstract_text_view`, `notes_text_view`; wrap word, accepts-tab False, `labelled-by` their group |
| `Gtk.Button` (copy abstract/notes) | 2 | `flat` | `on_copy_abstract_clicked`, `on_copy_notes_clicked` | `edit-copy-symbolic`; copy the selection if there is one, else the whole field |
| `CitationPreview` | 1 | — | `entry` bound to the page's `entry` | see its own section |
| `Adw.ActionRow` | 1 | — | `entry-page.set-pdf` | "Add PDF", activatable, `go-next-symbolic` suffix image |
| `Gtk.Button` (open PDF) | 1 | `flat` | `entry-page.open-pdf` | `Adw.ButtonContent`, `external-link-symbolic`, "_Open" |
| `Gtk.Button` (trash PDF) | 1 | `destructive-action` | `clicked`→`on_trash_pdf_button_clicked` | `user-trash-symbolic`, tooltip "Delete PDF" |
| `Gtk.Picture` | 1 | `card` | — | `pdf_picture`, keep-aspect-ratio, overflow hidden, `labelled-by` `pdf_preview` |
| `Gtk.Image` + `Gtk.Label` (preview error) | 1 + 1 | `title-1` (label) | — | `image-missing-symbolic` at 128px, "Could not set PDF Preview" |

Repeater: `extras_list_box` binds a `Gtk::StringList` of the entry type's extra
fields, one `Adw.EntryRow` per field titled by `utils::translated_key`
(Author, Year, Title, Volume, Number, Pages, Publisher, Journal, Address, How
Published, Note, Book Title, Series, Archive Prefix, ePrint, Primary Class,
Month, Editor, Organization, School, Institution, ISSN, ISBN, url). The
`eprint` row grows a `flat` `external-link-symbolic` button (tooltip "Open in
Browser") that is sensitive only when `eprint_url` parses.

Conditional: `other_entry_type_row` only when the type is Other; `extras_box`
only when the type has extra fields; `abstract_preferences_group` ↔
`notes_preferences_group` chosen by `EntryType::uses_abstract`; `pdf_stack`
switches on whether `~/Documents/Bibliography/<key>.pdf` exists;
`preview_error_stack` switches to the error box when Poppler rendering fails.
Dynamic CSS: `error` added to / removed from `citation_key_entry` when an
applied citation key collides.

Behaviour: bidirectional `BindingGroup` bindings for author, title (both
whitespace-collapsing), year, doi, entry-type, abstract buffer and notes
buffer; PDF preview rendered from page 0 with Poppler onto a Cairo surface,
re-rendered on `notify::scale-factor`; renaming the key moves the PDF file;
actions `entry-page.open-pdf` and `entry-page.set-pdf`.

## data/resources/ui/citation_preview.ui + src/citation_preview.rs — the formatted citation

→ `lib/citation_preview.rb` · **present**

| Widget | × | CSS | Signals / actions | Notes |
|---|---:|---|---|---|
| `Adw.Bin` (template) | 1 | — | `notify::template`→`on_template_notify` | properties `entry`, `template-kind`, `format-kind`, read-only `template` |
| `Gtk.Revealer` | 1 | — | — | `template_revealer`, slide-up, revealed only for `Template::Custom` |
| `Adw.PreferencesGroup` | 2 | — | — | "Custom Citation Format", "Citation" |
| `Gtk.MenuButton` | 1 | `flat` | — | `dialog-question-symbolic`, tooltip "Help", opens the help popover |
| `Gtk.DropDown` | 2 | — | bound to `template-kind` / `format-kind` and to the `template-kind` / `format-kind` settings | `GtkEnumList` over `Template` (MLA, AMS, APA, Chicago, Harvard, Vancouver, Custom…) and over `Format` (Plain, Markdown, LaTeX); `controls` `source_view` |
| `Gtk.TextView` | 1 | `inline` | buffer `changed` writes the custom template back to GSettings | `text_view`, monospace, wrap word, in a 90px `card` scroller |
| `GtkSource.View` | 1 | `inline` | — | `source_view`, read-only, wrap word, no tab; buffer language swapped per format (none / markdown / latex), style scheme follows `Adw::StyleManager` dark |
| `Gtk.Overlay` + `Gtk.Button` | 1 + 1 | `flat` (button) | `clicked`→`on_clicked` | `edit-copy-symbolic`, tooltip "Copy", margins 6, end/end; copies the rendered citation and toasts "Copied" |

Help popover (a top-level `GtkPopover`, not inside the template):

| Widget | × | CSS | Notes |
|---|---:|---|---|
| `Gtk.Popover` | 1 | — | `popover` |
| `Gtk.Label` | 3 | 2 × `heading` | "Custom Template"; the `%SOMETAG%` explanation (markup, 4 lines, 40 chars); "Example:" |
| `Gtk.ScrolledWindow` | 1 | `card` | 300×70, `hscrollbar-policy=never` |
| `Gtk.TextView` | 1 | `inline` | `help_source_view`, read-only monospace showing the AMS article template |

The 14 built-in templates (7 styles × article/book) and the per-type
article/book decision live here. `custom-book-template` and
`custom-article-template` are persisted in GSettings.

## data/resources/ui/new_entry_dialog.ui + src/new_entry_dialog.rs — Add new Entry

→ `lib/new_entry_dialog.rb` · **present**

An `AdwDialog`, 360×360, content-height 270, title "Add new Entry",
default widget `create_button`, initial focus `entry_row`.

| Widget | × | CSS | Signals / actions | Notes |
|---|---:|---|---|---|
| `Adw.ToolbarView` + `Adw.HeaderBar` | 1 + 1 | — | — | `show-end-title-buttons=False`, `show-title=false` |
| `Gtk.Button` (cancel) | 1 | — | `clicked`→`on_cancel_clicked` | "_Cancel", can-shrink |
| `Gtk.Button` (create) | 1 | `suggested-action` | `clicked`→`on_create_clicked` | "C_reate", starts insensitive, enabled only when the visible page has content |
| `Adw.ViewStack` | 1 | — | `notify::visible-child`→`on_stack_visible_child_notify` | pages `new`, `from_bibtex`, `from_doi` |
| `Adw.ViewStackPage` | 3 | — | — | "New" (`list-add-symbolic`), "From BibTeX" (`document-new-symbolic`), "From DOI" (`document-new-symbolic`) |
| `Adw.ViewSwitcherBar` | 1 | — | — | bottom bar, `reveal=True` |
| `Gtk.ListBox` | 1 | `boxed-list` | — | selection-mode none |
| `Adw.EntryRow` (key) | 1 | — | `notify::text`→`on_entry_notify_text` | "Citation Key", activates-default |
| `Adw.ComboRow` | 1 | — | — | "Citation Type", `GtkEnumList` over `EntryType` with `entry_type_name`, `selected=1` (Article) |
| `GtkSource.View` | 1 | `newentrydialog` | buffer `changed` → re-check the create button; `Gtk.EventControllerFocus` `enter` clears the placeholder once | bibtex language, syntax highlighting, no bracket matching; placeholder "Insert a BibTeX citation here" + a sample `@book{lamport1985i1,…}`; style scheme follows dark mode |
| `Adw.PreferencesGroup` + `Adw.EntryRow` (DOI) | 1 + 1 | — | `notify::text`→`on_doi_notify_text` | "DOI", activates-default; resolving goes through doi2bib and rewrites the key as `<lastname><year><first 6 of title>`, sanitised |

`newentrydialog` is an app-defined class from `style.css`.

## data/resources/ui/add_pdf_dialog.ui + src/add_pdf_dialog.rs — Add PDF

→ `lib/add_pdf_dialog.rb` · **present**

An `AdwDialog`, 360×270 request, content 450×270, title "Add PDF", initial
focus `pdf_url_entry_row`. Own signal `pdf-set`.

| Widget | × | CSS | Signals / actions | Notes |
|---|---:|---|---|---|
| `Adw.ToolbarView` + `Adw.HeaderBar` | 1 + 1 | — | — | bare header |
| `Adw.ToastOverlay` | 1 | — | — | toasts "Could not set PDF", "Could not download PDF" |
| `Gtk.Overlay` + `Gtk.ProgressBar` | 1 + 1 | `osd` | — | `progress_bar`, valign start, hidden until a download starts, then pulsed every 100 ms |
| `Gtk.ScrolledWindow` + `Adw.Clamp` + `Gtk.Box` | 1 + 1 + 2 | — | — | margins 24/24/12/12, spacing 12 |
| `Gtk.Label` (error) | 2 | `error` ×2 | — | `error_box`, hidden unless `~/Documents/Bibliography` is missing: "Bibliography folder not set" + "Make sure that {path} exists" |
| `Gtk.Label` (explanation) | 1 | `dimmed` | — | "Creates a copy of the selected pdf and stores it in your Bibliography folder." |
| `Gtk.ListBox` | 1 | `boxed-list-separate` | — | selection-mode none |
| `Adw.EntryRow` | 1 | `error` (dynamic, on failed download) | `entry-activated`→`on_entry_activated` | `pdf_url_entry_row`, "Add From URL", activatable |
| `Gtk.Button` | 1 | `flat` | `clicked`→`on_download_pdf_button_clicked` | `download_button`, `go-next-symbolic`, tooltip "Download…", goes insensitive while downloading |
| `Adw.ButtonRow` | 1 | — | `activated`→`set_from_file_row_activated` | "Add From File…", `document-open-symbolic`; a `Gtk::FileDialog` titled "Select a PDF" filtered to "PDF Files" (`application/pdf`) |

Downloads are validated by opening them with Poppler before being written to
`<Bibliography>/<citation_key>.pdf`; success emits `pdf-set` (the entry page
reloads its preview and the window toasts "PDF added") and closes the dialog.

## data/resources/ui/shortcuts.ui — the shortcuts dialog

→ `lib/shortcuts_dialog.rb` · **present**

A standalone named `AdwShortcutsDialog` with no backing source file; aliased in
the gresource as `shortcuts-dialog.ui` so libadwaita wires `app.shortcuts` to
it.

| Section | Items (action) |
|---|---|
| General | Show Shortcuts (`app.shortcuts`), New Window (`app.new-window`), Close Window (`window.close`), Quit (`app.quit`) |
| Bibliography | Open (`win.open('')`), Save (`win.save`), Search (`win.search`) |

All titles carry the `shortcut window` message context.

## src/selection.rs — the sidebar selection model

→ `lib/selection.rb` · **present**

Not a widget: a `GObject` implementing `Gio::ListModel` + `Gtk::SelectionModel`
(upstream GType name `SidebarSelection`) with properties `model`,
`selected-item` and `hide-selection`. It is what lets the sidebar show no
selection while the content pane is collapsed, and it tracks the selected item
across `items-changed`. `EntryList` is unusable without it.

## src/utils.rs — helpers behind the UI

→ `lib/utils.rb` · **present**

`translated_key` (the 24 field labels listed under EntryPage),
`build_google_schoolar_url`, `build_arxiv_url`, `bibliography_path` /
`bibliography_path_string` (`~/Documents/Bibliography`), `pdf_filename`,
`remove_recent_file`, `eprint_url` (bare arXiv ids → `https://arxiv.org/abs/…`),
`doi_url` (bare DOIs → `https://doi.org/…`), `has_pdf`, `open_pdf`
(`Gtk::FileLauncher`), `bib_from_doi`, `generate_key`, `sanitize_key`, and the
HTTP `get`. `generate_key` and `sanitize_key` carry upstream's only unit tests.

## Non-widget inputs

| Upstream | → port | Kind | Present | Notes |
|---|---|---|---|---|
| `data/org.gnome.World.Citations.gschema.xml.in` | `lib/settings.rb` | GSettings schema | yes | 8 keys: `window-width` 800, `window-height` 600, `is-maximized` false, `recent-files` `[]`, `custom-book-template`, `custom-article-template`, `template-kind` 0, `format-kind` 0 |
| `data/resources/style.css` | `lib/style.rb` | stylesheet | yes — inline, no gresource | `textview { padding: 6px }`, `.padded-sidebar`, inline-textview background reset, `textview.newentrydialog`, card-undershoot reset |
| `data/resources/resources.gresource.xml.in` | — | resource manifest | n/a | the port builds widgets in code; only the three bundled icons and `style.css` still need a home |
| `data/resources/icons/*.svg` | `data/icons/` | assets | yes — under `data/icons/hicolor/scalable/actions/` | `document-text-symbolic`, `external-link-symbolic`, `image-missing-symbolic` — all three are referenced by the UI above |
| `data/icons/org.gnome.World.Citations*.svg` | `data/icons/` | app icon | yes — `data/icons/hicolor/scalable/apps/` | used by both front-page status pages and `set_default_icon_name` |
| `data/org.gnome.World.Citations.metainfo.xml.in.in` | `data/` | appstream | yes — `data/org.rubygtkproject.Citations.metainfo.xml` | the about dialog is built `from_appdata` on it |
| `data/org.gnome.World.Citations.desktop.in.in` | `data/` | desktop entry | yes — `data/org.rubygtkproject.Citations.desktop(.in)` | — |
| `cratebibtex/src/bib.rs` | `lib/bib.rb` | model | yes | the `Gio::ListModel` of entries: parse, save, modified flag, add/remove/find, `key_exists` |
| `cratebibtex/src/entry.rs` | `lib/entry.rb` | model | yes | one citation: author, title, year, doi, citation-key, entry-type, x-abstract, notes, dynamic extra fields, `from_bibtex`, `serialize`, `format_citation` |
| `cratebibtex/src/entry_type.rs` | `lib/entry_type.rb` | enum + model | yes | 16 types (Other, Article, Book, Misc, In Proceedings, Unpublished, Online, Booklet, Conference, In Book, In Collection, Manual, Master's Thesis, PhD Thesis, Proceedings, Tech Report), their translated names, `uses_abstract`, and the per-type extra-field lists that drive the extras repeater |
| `cratebibtex/src/format.rs` | `lib/citation_format.rb` | enum | yes | Plain / Markdown / LaTeX, their names, and `format_authors` |
| `texer/src/lib.rs`, `texer/src/utils.rs` | `lib/texer.rb` | helper | yes | LaTeX → plain text; every author and title shown in the sidebar and every search URL goes through it |
| `po/` (42 locales, 21 POTFILES) | `po/` + `lib/i18n.rb` | translations | yes | gettext domain `citations`; `lib/i18n.rb` binds the domain and every string goes through `_`/`p_`; all 42 catalogues are present under `po/` |

---

## What is left

The order the port was written in is the dependency order the first survey
called for, and it held: models and helpers, then the selection model and the
sidebar, then the window, then the editor, the preview and the dialogs, then
the translation catalogues and the desktop entry. Every upstream component and
input now has a counterpart; what remains is parity work inside the files
that exist (see `component-parity` and `test-parity`), not an absent file.

## Known deviations from upstream, recorded as gaps

These are places where a component exists but does not do what upstream's
does. They are deviations, not omissions, and each is commented in the file
that carries it.

1. **GtkSourceView.** `citation_preview` and `new_entry_dialog` upstream use `GtkSource::View`/`Buffer` with the bibtex, markdown and latex languages and Adwaita style schemes. The port uses a plain `Gtk::TextView` in both, so the syntax highlighting is absent.
2. **Poppler.** `entry_page`'s PDF preview renders page one with poppler upstream; `add_pdf_dialog` validates a download by handing the bytes to poppler. The port leaves the preview picture empty and validates without poppler.
3. **`app.shortcuts` is implicit upstream.** libadwaita installs it from the aliased gresource; the port installs the action itself and presents `lib/shortcuts_dialog.rb`.
4. **No gresource.** The port builds every widget in code, so `style.css` is carried inline in `lib/style.rb` and the three symbolic icons are installed as files under `data/icons/`.
5. **Network from the main loop.** "From DOI" (doi2bib) and "Add From URL" fetch over HTTP; the Ruby path needs checking for main-loop safety under a slow or dead network.
