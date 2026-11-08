# Component identification — Citations (audit pass, 2026-11-08, round 4)

| | |
|---|---|
| Upstream | `origin/master` @ `e360482` |
| Port | `ruby` @ `265669c` |
| Runtime gate this round | `drive_ok=true`, 78 checks (`test/drive_main_window.rb`), 0 failed, 18 components claimed |

This pass re-ran the gate independently
(`nix develop --command xvfb-run -a ruby test/drive_main_window.rb`) and got
78/78, `DRIVE OK`, no `GLib-GObject-CRITICAL` in stderr — matching the stated
gate exactly. It also ran the other three entry points the repo carries
(`test/drive_devel_build.rb`, `test/test_models.rb`, `test/test_texer.rb`):
4/4, 93/93, 84/84, all green, none in the quoted 78.

## Round 3's two open findings are now fixed in code — independently re-verified, not re-asserted

Round 3 (`1297a7e`) left two things open: (1) `lib/citation_preview.rb`'s
`template_names` called `gettext` on a loop variable for the `Chicago` /
`Harvard` / `Vancouver` labels, so none of the three was ever extracted into
any catalogue; (2) the devel-profile branch (`Config::IS_DEVEL` true) was
correct on inspection but driven by no check on either side. Commit
`265669c`, which landed since, addresses both:

```
$ grep -n "when 'Chicago'" -A1 lib/citation_preview.rb
443:        when 'Chicago'
444:          _('Chicago')
$ grep -F '"Chicago"' po/citations.pot
msgid "Chicago"
```

`template_names` is now a `case`/`when` with a literal `_()` per branch,
mirroring upstream's `Template::to_translatable_string` match arms exactly.
`Chicago`, `Harvard` and `Vancouver` are confirmed present in
`po/citations.pot` and in `po/fr.po` (and all 43 locales — see
`TRANSLATION_PARITY.md`), not merely in the source.

`test/drive_devel_build.rb` is a new script (new in `265669c`) that forces
`Config::IS_DEVEL` to `true` before `require 'main'` and drives all three
gated sites: the window's `devel` CSS class, both status pages' app-id icon,
and the About dialog's raw `VERSION` string. Run fresh this pass: 4/4,
`DRIVE OK`. This is the one component this round's gate number (78,
`drive_main_window.rb` only) does not cover — it is a second, separate drive
script, not folded into the 78. Noted so "78/78" is not read as "every
driven check in the repo"; the full count across all four entry points this
pass ran is 78 + 4 + 93 + 84 = 259, all green.

## Third thing fixed: the metainfo file now has its own catalogue entries

Round 3's `TRANSLATION_PARITY.md` found `data/org.rubygtkproject.Citations.metainfo.xml`
missing from `po/POTFILES.in` entirely, so its `<summary>`, `<description><p>`
and `<caption>` were never extracted and the about dialog's `comments` string
translated only by accident (an independent, unrelated `Comment=` string in
the desktop file happening to be byte-identical). `265669c` adds the metainfo
file to `po/POTFILES.in` and regenerates every catalogue:

```
$ cat po/POTFILES.in | grep metainfo
data/org.rubygtkproject.Citations.metainfo.xml
$ grep -F "Manage your bibliography" po/citations.pot
msgid "Manage your bibliography"
$ grep -F "Main window" po/fr.po
msgid "Main window"
```

Both strings are present and independently extracted now, in every one of
the 43 locale files (confirmed for `fr.po`; see `TRANSLATION_PARITY.md` for
the full catalogue diff). The about dialog's `comments` call site
(`lib/about_dialog.rb:58`, `_(first_match(SUMMARY_RE, ...))`) is still a
non-literal argument the census cannot resolve on its own — see
`TRANSLATION_PARITY.md` and `ACCOUNTABILITY_ENSURANCE.md` for why that is
still worth naming even though the string it resolves to is now backed by a
real, independently-extracted msgid rather than a coincidence with an
unrelated file.

## What held up this pass (re-verified, not re-asserted)

- 78/78 `drive_main_window.rb` checks, 4/4 `drive_devel_build.rb`, 93/93
  models, 84/84 texer — all re-run fresh, zero stderr `CRITICAL` across all
  four.
- All 8 `gschema` keys/defaults/`<summary>` strings present and byte-identical
  (`data/org.rubygtkproject.Citations.gschema.xml.in` exists, unchanged this
  round).
- `GtkSource::View`/`Buffer` and `Poppler::Document` bindings unchanged and
  still exercised by drive checks 32 and 42.
- 22 upstream `#[test]` functions (recounted directly from `origin/master`'s
  five source files) against 29 `T.test(...)` blocks in the port — a
  superset; see `TEST_PARITY.md`.
- 43/43 `.po` files present on both sides (basename diff empty, unchanged).
- No `TODO`/`FIXME`/`not implemented`/`for now`/`in a future` in `lib/` or
  `exe/` (`grep -rniE` over both, clean, re-run this pass).

## New finding this round — a real, if low-impact, translation gap in the exact string the round-3 fix targeted

Fixing `Chicago`/`Harvard`/`Vancouver` did not fully close the catalogue for
`Template::to_translatable_string`. Upstream's bare `gettext("Custom")`
(`src/citation_preview.rs:55`) is extracted into every upstream `.po`, and
the port's catalogue has no `"Custom"` key at all — only `"Custom…"`. Tracing
every call site of `Template::to_translatable_string` upstream shows this
branch is unreachable dead code (the one caller that can hit `Template::Custom`
guards it and calls `gettext("Custom…")` instead), so no user-visible string
is missing in practice — but the key itself is a real catalogue gap per
`translation-parity`'s rule that there is no third state. See
`TRANSLATION_PARITY.md` for the full trace and `COMPONENT_PARITY.md` for the
component-axis entry.

## Verdict

Not clean. Both of round 3's open findings are fixed and independently
re-verified against the running code and the regenerated catalogues, not
against either side's prior prose. One new catalogue-completeness gap found
this pass, on the same axis the round-3 fix targeted: upstream's bare
`"Custom"` key (dead code upstream, but still a key 43 catalogues carry) has
no counterpart in the port's catalogue. See `TRANSLATION_PARITY.md` and
`ACCOUNTABILITY_ENSURANCE.md`.
