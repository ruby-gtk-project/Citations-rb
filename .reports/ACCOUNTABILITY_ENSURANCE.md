# Accountability ensurance — Citations (audit pass, 2026-11-08, round 4)

Re-ran `grep -rniE 'TODO|FIXME|not implemented|for now|in a future'` over
`lib/`, `exe/`, `.reports/`, and the repo root (`PORTING.md` does not exist
— confirmed again this pass, not assumed). Ran the skill's own
`scripts/find-excuses.sh` over the whole tree as well, which catches a wider
construction set (`by design`, `n/a`, `not applicable`, `out of scope`, etc.).

## What the excuse scan found, and why none of the three hits is a real finding

```
$ bash .claude/skills/accountability-ensurance/scripts/find-excuses.sh .
.reports/TRANSLATION_PARITY.md:111:by design: ...
.reports/component-inventory.md:60:n/a: ...
.reports/component-inventory.md:441:n/a: ...
```

Each was opened and read in context, per the rule that a construction match
is a lead, not a verdict, and the two carved-out exceptions the skill names
— a document *about* excuses, and a factual "n/a" that names real evidence
rather than asserting a feature cannot exist:

- `TRANSLATION_PARITY.md:111`, *"One string survives by coincidence, not by
  design"* — this sentence is pointing out the **absence** of a deliberate
  mechanism behind a working-by-luck call site, as a warning that it is
  fragile. It is the opposite of an excuse: it refuses to let the reader
  credit the code with intent it does not have. Not a hit.
- `component-inventory.md:60`, GtkBuilder template row marked `n/a` for the
  port — the row's own text names the reason: *"widgets are built in
  code"*, a structural fact about the house style (no `.ui` files exist on
  the port side by design of the whole campaign, documented in the
  `ruby-gtk` skill, not invented here to excuse this app), and the adjacent
  columns in the same table show the actual widget/CSS/signal counts were
  still compared. No behaviour is being waved away.
- `component-inventory.md:441`, resource-manifest row marked `n/a` — same
  shape: the row states plainly what still needs a home (three icons,
  `style.css`) and confirms they are present elsewhere in the same cell.
  Nothing is excused; the manifest format itself (a GResource XML index) has
  no referent in a port that does not build gresources, and the row says so
  while still accounting for every asset it would have listed.

None of the three converts a missing behaviour into settled architecture.
No rewrite performed — rewriting would be adding a construction where there
is no actual excuse, which would be the inverse failure (obscuring real
clean prose with insult-bank boilerplate it does not deserve).

## Main finding — this round's two fixes are independently verified, not just read as fixed in the previous round's reports

Round 3's `ACCOUNTABILITY_ENSURANCE.md` called out two prior-round reports
for asserting an open gap for five hours after the code closed it, with
nobody re-checking the claim against the code. This round's discipline was
applied to itself: every claim in this round's five reports that a round-3
finding is now fixed is backed by a command run this pass against the
regenerated artifact (`po/citations.pot`, `po/fr.po`, a fresh
`catalogue.rb --compare` run, a fresh `drive_devel_build.rb` execution) —
not by reading `265669c`'s commit message and trusting its own description
of what it did. Specifically:

- The `Chicago`/`Harvard`/`Vancouver` fix is confirmed by `grep -F '"Chicago"'
  po/citations.pot` and by the catalogue compare script reporting `state:
  ported` with real sites on both sides — not by reading the new
  `case`/`when` in `lib/citation_preview.rb` and assuming it worked.
- The metainfo fix is confirmed the same way, against `po/citations.pot` and
  `po/fr.po` directly.
- The devel-branch coverage fix is confirmed by actually running
  `test/drive_devel_build.rb` under `nix develop --command xvfb-run -a
  ruby` and reading `4 of 4 checks passed` / `DRIVE OK` from its own output
  — not by reading the script's source and concluding it would pass.

## Second finding — the fix commit's own catalogue work left one new, real gap and this round names it rather than accepting "fixed" as all-or-nothing

`265669c`'s commit message describes itself as fixing the untranslatable
template names. Taking that at face value and closing the book there is
exactly the failure mode round 3 found in round 2: trusting a
self-description instead of re-running the artifact it describes. This
round's fresh catalogue diff (not read from the commit, generated fresh
against both trees) surfaces that the fix closed 3 of upstream's 4
`Template::to_translatable_string` literals and missed the fourth (bare
`"Custom"`, dead code upstream, but still a key 43 catalogues carry). That
is recorded as a gap in `TRANSLATION_PARITY.md` and `COMPONENT_PARITY.md`,
not folded into "the template-text gap is fixed" as a rounding error.

## Third finding — two of this pass's own scanner hits were verified false before being written down, so they could not become the next round's false green

The catalogue compare script this pass reported two additional (gap, extra)
message pairs beyond the real `"Custom"` gap. Writing those down as gaps
without opening the actual `.po` files — the exact step round 2's
`TRANSLATION_PARITY.md` skipped for the `Chicago` claim — would have
produced two new false findings in this round's own ledger. Both were
checked against `origin/master`'s `po/fr.po` and the port's, by hand, before
being classified, and both turned out to be scanner artifacts (a `//`
comment line breaking the upstream match, an un-decoded XML entity in a
`.ui` file) rather than real divergence. Recorded in `TRANSLATION_PARITY.md`
as named scanner limitations, per the skill's "fix the scanner, or state the
discrepancy" rule, rather than silently dropped or silently reported as
gaps.

## What this round verified rather than re-asserted

- The runtime gate: re-ran all four entry points fresh
  (`drive_main_window.rb`, `drive_devel_build.rb`, `test_models.rb`,
  `test_texer.rb`) rather than trusting any number from a prior report —
  259/259, independently, zero stderr `CRITICAL`.
- The translation catalogue: regenerated both sides' catalogues with the
  skill's own scripts and diffed them fresh, rather than reading
  `TRANSLATION_PARITY.md`'s prose from the previous commit.
- Every "fixed" claim in this round's own reports: backed by a command run
  this pass against the artifact the claim is about, named above.

## Verdict

Not clean, and for a reason sharper than "one gap remains": the fix commit
this round audits is real and independently verified — it is not a false
green — but it was not complete, and this round's job was to find the
remainder rather than accept the commit message's own scope. One real,
narrow translation gap stands (upstream's dead-code `"Custom"` key). Two
near-misses in this round's own scan were checked and dismissed correctly
rather than written up as gaps. No excuse language found in `lib/`, `exe/`,
or the reports; the three grep hits in `.reports/` are factual statements
about structural absence, not sanctioned omissions, and were left as
written.
