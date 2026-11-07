# Translation parity — Citations (audit pass, 2026-11-07)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `b9516f9` |
| Upstream `.po` files | 43 (`po/*.po`), matching `LINGUAS` exactly |
| Port `.po` files | 43 — `diff` of the basename lists is empty |
| Upstream messages (rough re-census, this pass) | ~154–159 distinct keys (154 by direct `.po` msgid count on `en_GB.po`, excluding the header and `msgid_plural` rows; this pass's regex census over `.rs`/`.ui`/gschema/metainfo/desktop found 159 before excluding `<releases>` content, which is correct — release notes are not extracted per gettext's ITS rules) |
| Port messages reached by this census | 144 literal-argument calls found; 7 confirmed absent; the remaining difference is accounted for below (not a gap) |

Note: no `scripts/catalogue.rb`/`msgid-census.sh` exist in this fork (the
skill's reference tooling is not vendored here), so this pass wrote an
equivalent ad hoc extractor (Python regex over `gettext(`/`ngettext(`/
`pgettext(` on the upstream side and `_(`/`n_(`/`s_(`/`p_(` on the port
side) rather than skip the census. The numbers below are a lead, confirmed
by reading every flagged row's source, not a machine-verified catalogue
diff — a real `catalogue.rb` run would be more exact and should replace
this the next time the skill's scripts are available in this fork.

## Confirmed gap — 7 messages with no port counterpart, anywhere

All 7 come from `data/org.gnome.World.Citations.gschema.xml.in`, which has
no file at all on the `ruby` branch (see `COMPONENT_IDENTIFICATION.md`):

| msgid | Upstream site |
|---|---|
| `Window width` | gschema `window-width` summary |
| `Window height` | gschema `window-height` summary |
| `Window maximized state` | gschema `is-maximized` summary |
| `List of paths of recent files` | gschema `recent-files` summary |
| `Default template for book citation` | gschema `custom-book-template` **and** `custom-article-template` summaries (upstream itself has a duplicate/wrong summary on the second key — not a port bug, noted for completeness) |
| `Default template kind for citations` | gschema `template-kind` summary |
| `Default template format for citations` | gschema `format-kind` summary |

`grep -rn` for each exact string under `lib/` returns zero hits. These are
real gaps: `lib/settings.rb` reimplements the 8 keys and their defaults
correctly but carries no translation marker on any of them, because there
is nothing to mark — the strings were never typed into the port at all.

## Not a gap, but flag it — `_(name)` on 7 template-kind labels

`lib/citation_preview.rb` builds its `TEMPLATES` table with literal names
(`'Mla'`, `'Ams'`, `'Apa'`, `'Chicago'`, `'Harvard'`, `'Vancouver'`,
`'Custom'`) and calls `_(name)` with the variable, not the literal, at the
display site (`lib/citation_preview.rb:404`). Each of those 7 literal
strings is byte-identical to upstream's `gettext("Chicago")` etc.
(`src/citation_preview.rs:51-55`), and the `.po` catalogues carried over
verbatim already contain translations for all 7, so **the app translates
correctly at runtime** — this is not a functional gap.

It is, however, exactly the "unresolved marker" case the skill calls out:
a real `xgettext`/census tool run against `lib/citation_preview.rb` alone
would never discover these 7 keys, because the argument to `_()` is a
variable, not a string literal. If this port ever needs to regenerate its
own `.pot` from source (rather than carrying upstream's forward forever),
these 7 keys silently fall out unless someone censuses `TEMPLATES` by hand
first. Recorded here so the next person doesn't have to rediscover it.

## Held up under spot-check

- Locale set: exact match, 43/43, confirmed by `diff` of basenames.
- The custom-template help string (`lib/citation_preview.rb`'s
  `help_label`) is byte-identical to `citation_preview.ui`'s translatable
  label once the `.ui` file's `&lt;tt&gt;`/`&lt;/tt&gt;` XML entities are
  decoded to `<tt>`/`</tt>` — an early read of this pass flagged it as a
  gap and that was a false positive from not decoding entities; recorded
  here so it is not re-flagged.
- `Chicago`/`Harvard`/`Vancouver`/`Custom`/`General`/`Bibliography`/`Open`/
  `Save`/`Quit`/`Close Window`/`Show Shortcuts` are all present and correct
  in `lib/shortcuts_dialog.rb` via `p_('shortcut window', '…')`, matching
  upstream's `pgettext("shortcut window", …)` context — an early pass of
  this census also flagged these as gaps before the `p_(` extraction was
  added to the script; also a false positive, also recorded so it is not
  re-flagged.

## Verdict

Not clean. 7 confirmed gaps (the dropped gschema file's summaries), plus one
flagged-but-functional unresolved-marker risk (the 7 `_(name)` template
labels). Every other string checked this pass — including two early false
positives from an incomplete first extraction — held up as byte-identical.
A full machine census with the skill's own `catalogue.rb`/`msgid-census.sh`
(not present in this fork) would be more exact than this pass's ad hoc
regex and should be run once those scripts exist here.
