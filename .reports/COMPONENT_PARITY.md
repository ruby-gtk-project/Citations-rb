# Component parity — Citations (audit pass, 2026-11-08, round 4)

| | |
|---|---|
| Upstream | `origin/master` @ `e360482` |
| Port | `ruby` @ `265669c` |
| Runtime gate this round | `drive_ok=true`, 78/78 checks (+4/4 devel, +93/93 models, +84/84 texer), 0 failed, 18 components claimed |

This round re-verifies round 3's one open finding (the template-text gap on
`lib/citation_preview.rb`) against the fix commit rather than re-opening it
blind, and re-runs every gate fresh.

## The template-text gap — re-checked against code and catalogue, confirmed fixed, with one residual

| Widget | Text | Verdict |
|---|:-:|---|
| Template dropdown item "Chicago" | ✓ | `_('Chicago')`, literal, in a `case`/`when` branch — extracted into `po/citations.pot` and every locale |
| Template dropdown item "Harvard" | ✓ | same shape, confirmed extracted |
| Template dropdown item "Vancouver" | ✓ | same shape, confirmed extracted |
| Template dropdown item "Custom…" | ✓ | `_('Custom…')`, literal, unchanged from round 3 |
| Template dropdown items "MLA"/"AMS"/"APA" | — | upstream does not call `gettext` on these either; `.upcase` is a correspondence |

Round 3's gap is closed: all three previously-untranslatable labels are now
literal `_()` calls and are confirmed present in `po/citations.pot` and
`po/fr.po` (not just read from source — see `TRANSLATION_PARITY.md`'s
catalogue diff). This is the component this round's `COMPONENT_IDENTIFICATION.md`
calls out as the headline fix.

**Residual, found by running a fresh upstream/port catalogue diff for this
component rather than re-trusting the fix's own commit message:** upstream's
`Template` enum also has a bare `gettext("Custom")` (no ellipsis), extracted
into every upstream `.po`, that the port's catalogue does not carry under any
key. Tracing its one call site (`Template::to_translatable_string`, called
only by `template_type_name`, which is guarded so `Template::Custom` always
takes the `"Custom…"` branch instead) shows the upstream string is itself
unreachable — no screenshot of either app will ever show "Custom" bare. So
this is a catalogue-completeness gap on the text axis, not a rendering
difference: the widget, in the running app, is correct, but the message set
the port emits is missing one upstream key. Recorded as a gap because the
skill defines no third state for "the key is dead code" — see
`TRANSLATION_PARITY.md` for the full trace.

## The devel-profile distinction — now driven, not just read

Round 2 found this a gap; round 3 confirmed the code fixed but the branch
unexercised by any check. `265669c` adds `test/drive_devel_build.rb`, which
forces `Config::IS_DEVEL` true and drives all three sites. Re-run fresh this
pass: 4/4, `DRIVE OK`.

| Axis | Upstream (`IS_DEVEL`) | Port (`Config::IS_DEVEL`) | Driven? | Verdict |
|---|---|---|---|---|
| Window CSS | `devel` class added in `constructed()` | `lib/window.rb:53-55` | ✓ `drive_devel_build.rb` check 0 | **parity** |
| Empty-state icons | both status pages' `icon-name` = app id, devel only | `lib/window.rb:229`, `:256` | ✓ checks 1-2 | **parity** |
| About dialog version | explicit `VERSION`, devel only | `lib/about_dialog.rb:103` | ✓ check 3 | **parity** |

This closes round 3's standing coverage note: the devel branch is no longer
"correct on inspection, unexercised by any check" — it is now driven and
asserted on, independently of the main 78-check gate (a separate script, not
folded into it — see `COMPONENT_IDENTIFICATION.md`).

## The metainfo catalogue gap — fixed

Round 3 found `data/org.rubygtkproject.Citations.metainfo.xml` absent from
`po/POTFILES.in`, so its `<summary>`/`<description>`/`<caption>` were never
extracted. `265669c` adds it and regenerates every catalogue; confirmed this
pass against `po/citations.pot` and `po/fr.po` directly (see
`TRANSLATION_PARITY.md`). The one thing this fix does not touch: the About
dialog's `comments` field (`lib/about_dialog.rb:58`) still reaches its
string through `_(first_match(SUMMARY_RE, ...))`, a non-literal argument the
scanner flags `!unresolved` on both rounds. It now resolves to a
byte-identical, independently-extracted msgid (`"Manage your bibliography"`)
rather than one borrowed by coincidence from the desktop file — a real
improvement in robustness — but the call site itself is unchanged and is
still worth naming as fragile: a future edit to either string breaks
translation silently, with nothing in the code to catch it.

## What held up this round (re-verified)

- All 8 `gschema` keys, defaults, `<summary>` strings — unchanged, re-checked.
- Add PDF dialog, `EntryList#filter`, action/accelerator/shortcut inventory,
  `GtkSource`/`Poppler` bindings, background-thread network pattern — all
  unchanged since round 3, re-spot-checked against the current tree (no
  commit since `d989da7` touched any of these files except `citation_preview.rb`).

## Verdict

Not clean. Round 3's template-text gap is fixed for three of the four
strings it named, confirmed against the regenerated catalogue, and the
devel-profile branch is now driven, closing round 3's coverage note.
One new, narrow gap found this pass: upstream's dead-code `"Custom"` (bare)
key is absent from the port's catalogue. Everything else re-verified holds.
