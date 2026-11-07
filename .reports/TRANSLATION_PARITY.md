# Translation parity — Citations (audit pass, 2026-11-07, round 3)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `d989da7` |
| Upstream `.po` files | 43 (`LINGUAS` diff with the port: empty) |
| Port `.po` files | 43 — basenames identical |

## Confirmed gap, live today — three citation-template names are untranslatable in the port

Round 2's report named this as "a flagged-but-functional risk... the app
translates correctly at runtime today because the `.po` files already
carry these 7 keys from upstream's own `gettext("Chicago")` etc. calls,
carried forward verbatim." **That claim is checked against the port's own
`po/fr.po` this pass and is false.** The port's catalogues do not carry
these keys — they were never extracted, because the port's own call site
is the same kind of non-literal argument that would prevent extraction
upstream too, and upstream avoids exactly that by calling `gettext` on a
literal in each branch.

Evidence, read both ways:

```
# upstream: src/citation_preview.rs, Template::to_translatable_string
Template::Chicago   => gettext("Chicago"),
Template::Harvard   => gettext("Harvard"),
Template::Vancouver => gettext("Vancouver"),
Template::Custom     => gettext("Custom"),
```
Each is a literal argument — `xgettext` extracts all four into
`po/citations.pot`, and all 43 locales carry (translated or not) msgids for
them. `po/fr.po` on `origin/master` has `msgid "Chicago"`, `msgid
"Harvard"`, `msgid "Vancouver"` present (confirmed directly, see below).

```ruby
# port: lib/citation_preview.rb#template_names
TEMPLATES.map do |name, _article, _book|
  if name == 'Custom'
    _('Custom…')                         # literal — extracted correctly
  elsif %w[Mla Ams Apa].include?(name)
    name.upcase                           # correspondence: upstream doesn't
                                           # gettext these either
  else
    _(name)                               # name is the LOOP VARIABLE here —
  end                                      # never extracted, for "Chicago",
end                                        # "Harvard", "Vancouver"
```

Direct comparison, this pass, msgid-by-msgid, between upstream's `po/fr.po`
(`git show origin/master:po/fr.po`) and the port's `po/fr.po`:

```
missing in the port's po/fr.po (present in upstream's):
  'Chicago'
  'Custom'
  'Harvard'
  'Vancouver'
  ...(see metainfo finding below for the rest)
```

`grep -F '"Chicago"' po/citations.pot` — **no match** — the port's own
template confirms these three are absent from the catalogue entirely, not
merely untranslated in French specifically. `'Custom'` (bare, no ellipsis)
is also absent, but the port's only runtime use of that path is
`_('Custom…')`, a different string that *is* in the catalogue — so `Custom`
bare is a latent gap (no current call site uses it) while `Chicago`,
`Harvard`, `Vancouver` are live: `template_names` is the dropdown
builder, called every time the template picker renders.

**Effect:** in any of the 42 non-English locales the port ships, the
template-kind dropdown shows the English words "Chicago", "Harvard",
"Vancouver" instead of the localized term upstream shows. This is not a
cosmetic nit; it is a visible, every-session string the user interacts
with on the one page (the citation editor) that round 2's own component
scan already covers.

## Second gap — the metainfo file is not in the port's `po/POTFILES.in` at all

Upstream's `po/POTFILES.in` includes
`data/org.gnome.World.Citations.metainfo.xml.in.in`. Its `<summary>`,
`<description><p>` and `<screenshot><caption>` elements are translatable
appstream content, extracted by the same build and present as msgids in
every upstream `.po` file: `"Main window"` (the screenshot caption) and
`"Manage your bibliographies using the BibTeX format."` (the long
description) are both present in upstream's `po/fr.po`, both absent from
the port's.

The port's `po/POTFILES.in` has 16 entries and does **not** include
`data/org.rubygtkproject.Citations.metainfo.xml`:

```
$ cat po/POTFILES.in
data/org.rubygtkproject.Citations.desktop.in
data/org.rubygtkproject.Citations.gschema.xml.in
lib/*.rb (14 files)
```

So the metainfo's own strings are never extracted, and the file's content
is static, English-only appstream metadata in every locale — this affects
the GNOME Software / Flathub listing (summary, screenshot caption, long
description) in a way a running-app screenshot review would never catch,
because the port's `lib/about_dialog.rb` reads the file by regex at
runtime and does not render the long `<p>` description or the screenshot
section at all (confirmed: `about_dialog.rb` only reads `NAME_RE`,
`SUMMARY_RE`, `DEVELOPER_NAME_RE` — no description, no screenshots). This
makes it a catalogue-completeness gap, not an in-app visible one — tracked
here because the ledger this skill keeps is the catalogue, not the
component.

One string survives by coincidence, not by design, and is worth naming so
it is not mistaken for evidence the mechanism works: `about.comments =
_(first_match(SUMMARY_RE, 'Manage your bibliography'))` calls `_()` on a
value read from the file at runtime (non-literal, same failure mode as
`Chicago` above) — it only translates correctly today because the
identical English string `"Manage your bibliography"` is independently
extracted from `data/org.rubygtkproject.Citations.desktop.in`'s `Comment=`
line, which *is* in `POTFILES.in`. If the metainfo's `<summary>` and the
desktop file's `Comment=` ever diverge by a single character, this call
silently stops translating, with no indication anywhere in the code that
it was ever relying on the coincidence.

## What held up this round (re-verified)

- Locale set: 43 upstream, 43 port, identical basenames.
- `data/org.rubygtkproject.Citations.gschema.xml.in`'s 7 `<summary>`
  strings — present in `po/citations.pot`, byte-identical, confirmed again
  this pass.
- `shortcuts.ui` ↔ `shortcuts_dialog.rb`'s `p_('shortcut window', …)`
  context strings — re-spot-checked, hold up.
- `.rb` files' own literal `_()`/`n_()` calls outside the two gaps above —
  spot-checked against `lib/entry_list.rb`, `lib/entry_page.rb`,
  `lib/window.rb`, no unmarked or reworded string found.

## Verdict

Not clean. Two confirmed gaps, both newly substantiated this pass against
the port's actual `.po`/`.pot` files rather than against a prior report's
prose: (1) `Chicago`/`Harvard`/`Vancouver` (and latently, bare `Custom`)
are absent from every port catalogue and show in English in all 42
non-English locales, live today, not a future risk; (2) the metainfo
file's own translatable strings (`<summary>`, `<caption>`, the long
`<description>`) are not in `po/POTFILES.in` and are never extracted,
a catalogue-completeness gap the running app's own UI mostly does not
expose, with one accidentally-working exception named above. Round 2's
report characterized finding (1) as functionally fine; it was not fine,
it had simply never been checked against the port's own `.po` output.
