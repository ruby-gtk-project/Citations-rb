# Component identification — Citations (audit pass, 2026-11-07, round 3)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `d989da7` |
| Runtime gate this round | `drive_ok=true`, 255 checks (78 drive + 93 models + 84 texer), 0 failed, 14 components claimed |

This pass re-ran the gate independently (`nix develop --command xvfb-run -a
ruby test/drive_main_window.rb`, `test/test_models.rb`, `test/test_texer.rb`)
and got 78/78, 93/93, 84/84 — 255 total, matching the stated gate exactly —
with no `GLib-GObject-CRITICAL` line in stderr. This is not a renumbering:
the drive script's checks 33-42 actually open the Add PDF dialog, reject a
bad URL, cancel, then reopen it and drive a real loopback-HTTP download to
completion, asserting the PDF lands on disk and the preview texture is real.

## Finding — two of the last round's checked-in reports are stale, not current

`COMPONENT_IDENTIFICATION.md` and `COMPONENT_PARITY.md` as committed at the
head of `ruby` (prior to this pass) both asserted, as their lead and only
open finding, that the devel-profile distinction — the `devel` CSS class,
the two status-page icons, the About dialog's version string, all gated on
upstream's `IS_DEVEL` / the port's `Config::IS_DEVEL` — was dropped with no
code reading the constant at all.

That was true when those reports were written (commit `1ed4e33`, 15:29:23).
It stopped being true at `19d4cff` (16:36:15), over an hour before those same
report files were next touched (`7a7c438`/`31b14ee` at 20:26, `d989da7` at
21:15) — and none of those three later commits re-checked or corrected the
devel-gap section, even though `d989da7` edited the very same
`COMPONENT_PARITY.md` file for an unrelated paragraph two sections below it.

Verified directly against the current tree, not against any report:

```
$ grep -n "IS_DEVEL" lib/window.rb lib/about_dialog.rb
lib/window.rb:53:      if Config::IS_DEVEL
lib/window.rb:229:      if Config::IS_DEVEL
lib/window.rb:256:      if Config::IS_DEVEL
lib/about_dialog.rb:103:      if Config::IS_DEVEL
```

All three sites upstream gates — `win.add_css_class('devel')`, both status
pages' `page.icon_name = Config::APP_ID`, and the About dialog's explicit
`@version` override — are now conditioned on `Config::IS_DEVEL` in exactly
the shape upstream's `window.rs`/`show_about_dialog` use. This is fixed, and
has been fixed since `19d4cff`. The two report files calling it an open gap
with no other evidence than their own prior text were simply never
re-verified against the code they describe. This is corrected in this
pass's `COMPONENT_PARITY.md` and recorded as the headline finding in
`ACCOUNTABILITY_ENSURANCE.md`: a report that outlives the code state it
describes is a false claim the moment the code moves, whichever direction
it moves in.

## Second staleness — `component-inventory.md`'s item 5 is an unresolved question that is already answered in code

`component-inventory.md`'s "Known deviations" item 5 reads: *"Network from
the main loop. 'From DOI' (doi2bib) and 'Add From URL' fetch over HTTP; the
Ruby path needs checking for main-loop safety under a slow or dead
network."* — phrased as an open question. Reading the two sites it names:

```
lib/add_pdf_dialog.rb:238:      Thread.new do
lib/add_pdf_dialog.rb:264:      GLib::Idle.add do
lib/new_entry_dialog.rb:284:        Thread.new do
lib/new_entry_dialog.rb:287:            GLib::Idle.add do
```

Both do the download/HTTP fetch on a background `Thread`, and resume on the
main loop via `GLib::Idle.add` to touch widgets — the standard main-loop-safe
pattern, and the one the drive suite's checks 39-42 now exercise end to end
for the Add PDF path (a real loopback download). The "needs checking"
language is stale residue from before that pattern was written; it should
read as resolved, not as an open item. Not a functional gap, but another
instance of the same report-staleness failure as above.

## Held up this pass (re-verified, not re-asserted)

- 255/255 runtime checks, independently re-run, zero stderr CRITICAL.
- All 8 `gschema` keys/defaults/`<summary>` strings present and byte-identical
  (`data/org.rubygtkproject.Citations.gschema.xml.in` exists).
- `GtkSource::View`/`Buffer` and `Poppler::Document` are real bindings, not
  stand-ins — confirmed in `lib/citation_preview.rb`, `lib/new_entry_dialog.rb`,
  `lib/entry_page.rb`, `lib/add_pdf_dialog.rb`, and exercised by drive checks
  32 (preview rendering) and 42 (PDF texture after a real download).
  `.rspec` (dead RSpec config for a runner this repo no longer has) is gone.
- 22 upstream `#[test]` functions counted directly from `origin/master`
  (`cratebibtex/src/entry.rs`:1, `format.rs`:4, `lib.rs`:1, `src/utils.rs`:2,
  `texer/src/tests.rs`:14 = 22) against 29 `T.test(...)` blocks across
  `test/test_models.rb`/`test/test_texer.rb` (15+14) — a superset, with the
  extra 7 being added coverage (`bib::list_model`,
  `bib::round_trips_through_a_file`, `entry_type::fields_and_names`,
  `format::kinds`, `utils::urls_and_filenames`,
  `settings::defaults_match_the_upstream_schema`, plus one extra split of
  `entry.rs`'s single test into `entry::format`/`entry::serialize`), not a
  shortfall.
- 43/43 `.po` files present on both sides (basename diff empty).
- No `TODO`/`FIXME`/`not implemented`/`for now`/`in a future` in `lib/` or
  `exe/` (`grep -rniE` over both, clean).

## Verdict

Not clean. No new functional component gap found this pass — the one real
gap carried from round 2 (the devel-profile distinction) is confirmed fixed
in code. What is not clean is the reporting itself: two files committed on
`ruby` asserted an open gap that had already been closed for five hours
before the files were last edited, and a third's "known deviation" item is
phrased as an open question the code already answers. See
`ACCOUNTABILITY_ENSURANCE.md`.
