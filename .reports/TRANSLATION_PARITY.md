# Translation parity — Citations (audit pass, 2026-11-08, round 4)

| | |
|---|---|
| Upstream | `origin/master` @ `e360482` |
| Port | `ruby` @ `265669c` |
| Upstream `.po` files | 43 (`LINGUAS` diff with the port: empty) |
| Port `.po` files | 43 — basenames identical |
| Catalogue run this pass | `.claude/skills/translation-parity/scripts/catalogue.rb` against a fresh `git worktree` of `origin/master` and the live `ruby` tree, compared with `--compare` |

This pass ran the skill's own census and compare scripts fresh rather than
reading either side's `.po`/`.pot` by eye, which is what round 3's reports
did (and what caught round 2's false green). Raw numbers:
`upstream: 160 messages, 181 occurrences, 13 reused, 2 unresolved` /
`port: 161 messages, 180 occurrences, 10 reused, 7 unresolved`. Compare
summary: `ported: 158, gaps: 2, extra: 3, kind_mismatch: 0, uses_mismatch: 9,
languages: 43/43, domain_matches: true, parity: false`.

**Every one of the 5 state-changing rows (2 gaps, 3 extra) was opened and
read by hand before being recorded below — the scanner's raw count is a
lead, not a verdict, per the skill.**

## Confirmed fixed — the round-3 gap, re-checked against the regenerated catalogue, not against the fix commit's own message

Round 3 found `Chicago`, `Harvard`, `Vancouver` absent from every port
catalogue because `template_names` called `_(name)` on a loop variable.
`265669c` rewrote it as a `case`/`when` with a literal `_()` per branch.
This pass's fresh catalogue diff confirms all three are now `ported`:

```
upstream_sites: [src/citation_preview.rs:51/52/53]   (gettext("Chicago") etc., literal)
port_sites:     [lib/citation_preview.rb:439/441/443] (_('Chicago') etc., literal)
state: ported, kind: single/single, uses: 1 → 1, for all three
```

Also confirmed fixed: the metainfo catalogue gap. Round 3 found
`data/org.rubygtkproject.Citations.metainfo.xml` missing entirely from
`po/POTFILES.in`. `265669c` adds it; this pass's fresh diff shows both
`"Manage your bibliography"` (the `<summary>`) and `"Main window"` (the
screenshot `<caption>`) as `ported`, each with a real upstream and port
site, present in `po/fr.po` on both sides.

## New, real gap — upstream's bare `"Custom"` has no counterpart in the port's catalogue

```
id: Custom
state: gap
uses: {upstream: 1, port: 0}
upstream_sites: [src/citation_preview.rs:55]      # gettext("Custom"), no ellipsis
text_owed: Custom
```

Traced by hand: `Template::to_translatable_string` is the only place
upstream's bare `gettext("Custom")` lives, and its only caller
(`template_type_name`, `src/citation_preview.rs:264-271`) is written as:

```rust
if type_ == Template::Custom {
    gettext("Custom…")      // always taken for Custom
} else {
    type_.to_translatable_string()   // never reached for Custom
}
```

So the bare `"Custom"` branch is dead code upstream — no user of either app
ever sees it — but `xgettext` extracts literals regardless of reachability,
so all 43 upstream `.po` files carry the key and the port's do not. This is
a catalogue-completeness gap by the letter of the rule (there is no
third state for "the key is dead"), with zero visible effect: both apps
show `"Custom…"` in the dropdown today. Recorded as a gap, not waved
through, and recorded as low-impact rather than inflated into a visible
regression — both of those are facts about the same row.

## Two pairs the scanner flagged as gap+extra, verified by hand to be false positives — not gaps

The compare step reported two (gap, extra) pairs beyond the one above.
Both were opened and checked against the actual `.po` content before being
dismissed, per the skill's instruction to treat the census as a lead:

**1. The two "Unsaved Changes" dialog subtitles.** The scanner reported
`"Do you want to write all changes to the bibliography before opening the
new one?"` and `"Do you want to write all changes to the bibliography?"`
as upstream-absent / port-only. They are not: `src/window.rs:690` and
`:722` call `gettext(...)` on exactly these two literals, each inside a
block comment the scanner's line-joiner does not see past
(`// NOTE Dialog subtitle which informs...`). Both strings are present and
translated in 26-28 of the 43 `.po` files on each side (confirmed:
`grep -A2 "Do you want to write all changes" po/fr.po` finds it in the
port; `po/be.po`, `po/fr.po`, etc. on upstream). Port site:
`lib/window.rb:842`, `:874`, byte-identical. **Not a gap — a scanner miss
on the upstream side, caused by a `//` comment between `gettext(` and its
string argument.**

**2. The custom-template help text.** The scanner reported the help label
(`"Write a custom format for your citations... <tt>sometag</tt> ..."`) as a
gap/extra pair because upstream's `.ui` file XML-escapes the angle brackets
(`&lt;tt&gt;`) and the port's Ruby source writes them literally (`<tt>`).
Checked against `origin/master`'s own `po/fr.po`, which carries the
*unescaped* form (`<tt>sometag</tt>`, confirmed: GtkBuilder's translatable
extraction decodes XML entities before handing the string to `xgettext`) —
the real msgid upstream ships is identical to the port's. **Not a gap — a
scanner limitation (it does not decode `.ui` XML entities), reported here
per the skill's "fix the scanner, or state the discrepancy" rule rather
than silently reconciled.**

## Unresolved markers — 6 of 7 on the port side are the gettext wrapper's own definitions, not call sites

```
port totals.unresolved: 7
```
6 of the 7 are `lib/i18n.rb`'s own method *definitions* (`def _(msgid) = ...`,
`def p_(context, msgid) = ...`, `def n_(singular, plural, count) = ...` and
the three abstract methods they implement) — the scanner matches the
parameter name as if it were a literal argument, which it structurally
cannot be; these are the translation primitives themselves, not calls to
them. Not a real unresolved count.

The 7th is real and was already named in round 3:
`lib/about_dialog.rb:58`, `_(first_match(SUMMARY_RE, 'Manage your bibliography'))`
— a non-literal argument. It resolves today to a msgid
(`"Manage your bibliography"`) that is now independently extracted from the
metainfo file itself (round 3's fix), which is a real improvement — it no
longer relies on a coincidental match with an unrelated desktop-file string
— but the call site is still non-literal and the scanner still cannot
verify it without reading the file at runtime. Flagged again because the
skill asks for `totals.unresolved` to be driven to 0 or censused by hand
every round, not carried forward as accepted.

## What held up this round (re-verified against the generated catalogue, not against prior prose)

- Locale set: 43 upstream, 43 port, identical basenames, `languages_missing: []`.
- `domain_matches: true` — both catalogues report domain `citations`.
- `kind_mismatch: 0` — no message is plural on one side and single on the
  other.
- 158 of 160 upstream messages are `ported`; the `uses_mismatch` list (9
  entries) was read in full and every entry beyond the two scanner-miss
  pairs above is a structural difference `component-parity` owns (e.g.
  `Citations` 4→5, `New Window` 1→2), not a translation-axis finding — per
  the skill's rule that `uses_mismatch` opens a component investigation,
  never a translation row.

## Verdict

Not clean, but narrowly so and precisely bounded: 158/160 messages ported,
43/43 languages, domain matches, no kind mismatches. The one real gap is
upstream's dead-code `"Custom"` key — zero visible effect, still owed by
the letter of the rule. Two scanner-reported gap/extra pairs were opened,
read against both sides' actual `.po` content, and found to be false
positives (a `//`-comment line break and an un-decoded XML entity), named
here rather than silently dismissed. One non-literal call site
(`about_dialog.rb:58`) remains unresolved by the scanner and is tracked
again rather than assumed closed because its resolved string happens to be
correct today.
