# Component identification — Citations (audit pass, 2026-11-07)

Upstream: `origin/master`. Port: `ruby` @ `b9516f9`.

This repeats the survey already on file at `.reports/component-inventory.md`
(which is accurate as an inventory of what exists) and adds what that survey
does not check: whether each inventoried item actually has a counterpart file
on disk, and whether anything it calls "present" is backed by a running
check. Two misses found that the existing inventory's own wording obscures:

## Miss 1 — the GSettings schema has no file at all

`data/org.gnome.World.Citations.gschema.xml.in` is a real upstream file with
8 keys. `find data -iname '*gschema*'` on the `ruby` branch returns nothing.
`.reports/component-inventory.md` lists `lib/settings.rb` as "the GSettings
keys … present", which is true of the *key names and defaults* and false of
the *file* — there is no schema, compiled or otherwise, anywhere in the port.

That file carried 7 translatable `<summary>` strings (`Window width`,
`Window height`, `Window maximized state`, `List of paths of recent files`,
`Default template for book citation` ×2 keys, `Default template kind for
citations`, `Default template format for citations`). None of them appear
anywhere under `lib/` (`grep -rn` for each string: zero hits). See
`TRANSLATION_PARITY.md` for the census this is drawn from.

## Miss 2 — `lib/add_pdf_dialog.rb` has zero runtime coverage

The inventory lists `lib/add_pdf_dialog.rb` (330 lines, "Add PDF") as a
present component. `grep -n 'AddPdfDialog\|add_pdf' test/drive_main_window.rb`
returns nothing. The only drive-script line that touches this area is
`ok   the PDF stack offers to add one`, which checks that the *button exists*
on the entry page, not that the dialog it opens builds, accepts a URL/file,
or does anything the 330 lines claim. This is a component with a file and no
check — unverified, not done, per this round's own runtime-gate instruction.

## Miss 3 — a live GObject-CRITICAL on every window build

`lib/entry_list.rb#filter` builds
`Gtk::StringFilter.new(Gtk::PropertyExpression.new(Entry, nil, 'search-text'))`.
Every drive run prints, on every `Window.new`:

```
GLib-GObject-CRITICAL **: g_object_ref: assertion 'G_IS_OBJECT (object)' failed
	...
	from lib/entry_list.rb:177:in 'EntryList#filter'
```

`nil` is passed where `Gtk::PropertyExpression.new` expects an object (the
second positional argument). The 65 checks never fail because none of them
assert on stderr; the warning is silently tolerated on every window the app
or the driver creates. This is a real binding-usage bug, not cosmetic noise —
confirmed by reproducing it with `nix develop --command xvfb-run -a ruby
test/drive_main_window.rb` twice.

Everything else in `.reports/component-inventory.md` — the widget/CSS/signal/
action totals, the file mapping, the per-unit tables — was spot-checked
against upstream source (window.rs/window.ui, citation_preview.rs,
settings keys, shortcuts.ui) during this pass and held up; it is not
reproduced here to avoid duplicating a file already on disk.
