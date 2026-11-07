# Component parity — Citations (audit pass, 2026-11-07, round 3)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `d989da7` |
| Runtime gate this round | `drive_ok=true`, 255/255 checks, 0 failed, 14 components claimed |

This round re-verifies round 2's one open finding (the devel-profile gap)
rather than re-opening it blind, and re-runs the gate independently rather
than trusting the number. See `COMPONENT_IDENTIFICATION.md` for the full
account of why round 2's checked-in reports were stale on this point.

## The devel-profile distinction — re-checked against code, confirmed fixed

| Axis | Upstream (`IS_DEVEL`) | Port (`Config::IS_DEVEL`) | Verdict |
|---|---|---|---|
| Window CSS | `devel` class added to the window in `constructed()` | `lib/window.rb:53-55`: `win.add_css_class('devel') if Config::IS_DEVEL` | **parity** |
| Empty-state icons | both status pages' `icon-name` set to the app id, devel only | `lib/window.rb:229`, `:256`: `page.icon_name = Config::APP_ID if Config::IS_DEVEL` | **parity** |
| About dialog version | explicit `VERSION` string, devel only; else derived from metainfo | `lib/about_dialog.rb:103`: `if Config::IS_DEVEL` branches between `@version` and the metainfo-derived release version | **parity** |

`Config::IS_DEVEL` is still hardcoded `false` in `lib/config.rb` (a checkout
has no meson configure step, same as upstream's own `config.rs.in` without a
build profile) — that is a correspondence, not a gap: the constant now
*gates* real behaviour on both sides identically, which is what parity on
this axis means. No driver check flips `IS_DEVEL` to `true` and screenshots
the result, so the devel-on branch itself is unexercised by the runtime
gate — worth naming, since "255/255" says nothing about a branch that is
never taken by any check, but it is a coverage gap in the gate, not a
component gap: the code path exists and matches upstream's shape on
inspection.

## What held up this round (re-verified, not re-asserted from round 2's text)

- All 8 `gschema` keys, defaults, and `<summary>` strings — byte-identical,
  file present at `data/org.rubygtkproject.Citations.gschema.xml.in`.
- Add PDF dialog — now driven end to end on **both** branches: checks 33-38
  (bad URL → rejected, error style, cancel, still no PDF) and checks 39-42
  (new this round's verification: a real loopback HTTP download completes,
  the PDF lands on disk under the citation key, the stack switches off the
  add-one state, and the preview shows a real `Gdk::Texture`, not the error
  view). Round 2 flagged the happy path as unexercised; it no longer is.
- `EntryList#filter` — no CRITICAL on a fresh drive run this pass either;
  confirmed by watching stderr for the whole run.
- Full action/accelerator/shortcut inventory — complete, 1:1 (unchanged from
  round 2, re-spot-checked: `app.quit`, `app.new-window`, `win.open('')`,
  `win.save`, `win.search`, `window.close`, `app.shortcuts`).
- `GtkSource::View`/`Buffer` (bibtex/markdown/latex languages, Adwaita /
  Adwaita-dark scheme switched on `Adwaita::StyleManager`'s dark-notify) and
  `Poppler::Document` (page-one preview rendering, download validation) are
  real bindings in `lib/citation_preview.rb`, `lib/new_entry_dialog.rb`,
  `lib/entry_page.rb`, `lib/add_pdf_dialog.rb` — confirmed by reading the
  `require`s and the `GtkSource`/`Poppler` constant usage directly, not by
  trusting the commit message that introduced them.
- Network-bound work (`From DOI` in `lib/new_entry_dialog.rb`, `Add PDF`'s
  download in `lib/add_pdf_dialog.rb`) runs on a background `Thread` and
  resumes on the main loop via `GLib::Idle.add` before touching any widget —
  the correct pattern, and no longer an open question (see
  `COMPONENT_IDENTIFICATION.md`'s note on `component-inventory.md`'s stale
  "needs checking" item).

## New gap found this round — the text axis, `lib/citation_preview.rb`'s template names

`component-parity`'s Step 3 requires the text axis to be checked per
component, not deferred wholesale to `TRANSLATION_PARITY.md`. Checking it
for `lib/citation_preview.rb` (unit: the citation preview / template picker)
found a real, currently-live gap that round 2's `TRANSLATION_PARITY.md`
characterized as a forward-looking risk only — it is not. See
`TRANSLATION_PARITY.md` for the full evidence; summarized against this
component:

| Widget | Text | Verdict |
|---|:-:|---|
| Template dropdown item "Chicago" | ✗ | **gap**: `_(name)` on a variable never extracts; msgid absent from every port `.po`, including `fr.po` |
| Template dropdown item "Harvard" | ✗ | **gap**: same cause |
| Template dropdown item "Vancouver" | ✗ | **gap**: same cause |
| Template dropdown item "Custom…" | ✓ | called as `_('Custom…')`, a literal — correctly extracted |
| Template dropdown items "MLA"/"AMS"/"APA" | — | upstream does not call `gettext` on these either (`to_string()` literals); port's `.upcase` is a correspondence, not a gap |

In any non-English locale, a user choosing "Chicago", "Harvard" or
"Vancouver" from the template dropdown sees the English word, where
upstream's `gettext("Chicago")` (a literal call, correctly extracted into
every one of its 43 `.po` files) shows the localized term. This is a
component-text-axis gap in its own right, tracked against the message in
`TRANSLATION_PARITY.md`.

## Verdict

Not clean. Parity holds on every widget/CSS/signal/action axis this pass
re-checked, including the devel-profile gap round 2 left open, which is
now confirmed fixed. One new gap found on the text axis: three of the
seven citation-template names (`Chicago`, `Harvard`, `Vancouver`) are
untranslatable in the port today, not merely "a risk for a future `.pot`
regeneration" as round 2's `TRANSLATION_PARITY.md` claimed — that
characterization was checked against the port's own `po/fr.po` this pass
and found to be wrong. The remaining coverage item is the devel branch
itself: no driver check builds the app with `IS_DEVEL` true.
