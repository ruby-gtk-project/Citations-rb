# Accountability ensurance — Citations (audit pass, 2026-11-07, round 3)

Re-ran `grep -rniE 'TODO|FIXME|not implemented|for now|in a future'` over
`lib/`, `exe/`, `.reports/`, and the repo root (`PORTING.md` does not exist
— confirmed, not assumed, same as round 2). The only hits are inside the
`.reports/` files' own prose quoting this grep pattern or describing past
findings — no excuse language in `lib/` or `exe/`.

## The most serious finding this round — a false green in round 2's own `TRANSLATION_PARITY.md`

Round 2's `TRANSLATION_PARITY.md` wrote, about `lib/citation_preview.rb`'s
`_(name)` variable-argument call for the `Chicago`/`Harvard`/`Vancouver`
template labels:

> "the app translates correctly at runtime today because the `.po` files
> already carry these 7 keys from upstream's own `gettext("Chicago")` etc.
> calls, carried forward verbatim. The risk is only forward-looking..."

This pass checked that claim against the artifact it is actually about —
the port's own `po/fr.po` and `po/citations.pot` — rather than taking the
prior report's prose at face value, and the claim is false:

```
$ grep -F '"Chicago"' po/citations.pot
(no output)
```

`Chicago`, `Harvard` and `Vancouver` are not in the port's catalogue at
all, in any language, and have not been since this file was written —
nothing about this pass's run changed that fact, it only checked it. The
three labels show in English in every one of the port's 42 non-English
locales today. See `TRANSLATION_PARITY.md` for the full evidence and
`COMPONENT_PARITY.md` for the component-text-axis entry.

This is the shape of finding the skill asks to treat as most serious: a
report asserted a thing was fine on the strength of its own narrative
("carried forward verbatim," "functionally fine today") without the one
command — a grep against the port's own `.po`/`.pot` output — that would
have disproved it in seconds. It is also not a one-off: round 1 made the
same claim about this exact code (round 2's report says "unchanged from
round 1"), so the claim was carried through two full audit rounds without
either one checking the file it was a claim about.

## Second finding — two reports asserted an open gap for five hours after it closed, and were not corrected when later touched

`COMPONENT_IDENTIFICATION.md` and `COMPONENT_PARITY.md`, as committed at
`1ed4e33` (15:29:23), both led with the devel-profile distinction
(`Config::IS_DEVEL`) as the one open gap this port carries. Commit
`19d4cff` (16:36:15) fixed exactly that gap in `lib/window.rb` and
`lib/about_dialog.rb`. Three later commits touched the tree —
`7a7c438`/`31b14ee` (20:26, both titled around "Inventory upstream
components") and `d989da7` (21:15, which edited `COMPONENT_PARITY.md`
directly, for an unrelated paragraph) — and none of them revisited the
devel-gap section to confirm it was still open. It was committed as an
open finding straight through to the head of `ruby` until this pass read
`lib/window.rb` and `lib/about_dialog.rb` directly and found all three
sites already gated.

This is the gentler failure mode than the one above — the error is
pessimistic, not a false green, and nobody shipped a false "all clear" on
it — but it is the same root cause: a claim about code that was never
re-checked against the code once the code changed, surviving across
multiple commits that touched the very file carrying the claim. A reviewer
reading `COMPONENT_PARITY.md` at `d989da7` and trusting its "Not clean —
devel gap" verdict would spend time re-investigating an already-closed
finding, which is its own kind of cost.

`component-inventory.md`'s item 5 ("Network from the main loop... needs
checking") is the same pattern a third time, smaller: the `Thread.new` +
`GLib::Idle.add` pattern it asks about is already in both call sites
(`lib/add_pdf_dialog.rb`, `lib/new_entry_dialog.rb`) and already driven by
the suite. Nobody re-answered the question the file itself posed.

## What this round verified rather than re-asserted

- The runtime gate: re-ran all three entry points fresh
  (`test/drive_main_window.rb`, `test/test_models.rb`, `test/test_texer.rb`)
  rather than trusting "255/255" as a number. Got 78+93+84 = 255,
  independently, with no stderr `CRITICAL`.
- The devel-profile fix: read `lib/window.rb` and `lib/about_dialog.rb`
  directly rather than trusting either round 2's "gap" claim or an
  assumption that it must have been fixed since.
- The 22-upstream-test census: recounted `#[test]` occurrences directly
  from `origin/master`'s five source files rather than citing the
  previous round's "22/22" figure.
- The translation gap: diffed the port's own `po/fr.po` against upstream's
  `po/fr.po`, msgid by msgid, rather than reading `TRANSLATION_PARITY.md`'s
  prose and moving on.

## Verdict

Not clean, and the reason is sharper than "one gap remains": round 2
shipped a false green — a specific, checkable claim about the port's own
translation output that was wrong and was never checked against that
output across two audit rounds. Two more findings are reports left stale
after the code under them moved, in both directions (one claimed a closed
gap was still open; a third asked a question the code had already
answered). None of this round's reports take a prior round's verdict on
faith; each either reproduces the evidence fresh or names exactly where
the prior claim diverged from the code.
