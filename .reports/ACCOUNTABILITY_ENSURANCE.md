# Accountability ensurance — Citations (audit pass, 2026-11-07, round 2)

Re-ran `grep -rniE 'TODO|FIXME|not implemented|for now|in a future'` over
`lib/`, `exe/`, `.reports/`, and the repo root (`PORTING.md` does not exist
in this repo — confirmed, not assumed). The only hits are inside round 1's
own `ACCOUNTABILITY_ENSURANCE.md` text, quoting the grep pattern itself —
no excuse language in `lib/` or `exe/`. Also checked `n/a`, `by design`,
`out of scope`, `settled`, `tradeoff` — the one hit worth discussing is in
`lib/settings.rb`'s class comment, examined below.

## Round 1's finding, re-verified actually fixed (not just reworded)

Round 1 called out `lib/settings.rb`'s class comment for using "an install
step would be worse than the alternative" as a calm-sounding excuse for
dropping the gschema file and its 7 translatable strings entirely — not
just the runtime-keys substitution, which round 1 agreed was legitimate.

This is the one case in this audit where the *fix that mattered* is the
one the skill actually wants: `data/org.rubygtkproject.Citations.gschema.xml.in`
was ported in full (verified in `TRANSLATION_PARITY.md`), and
`lib/settings.rb`'s comment was rewritten to name the real, narrower cost
in the first person:

> "I did not port that: this class reads and writes the same eight keys
> from a small JSON file under XDG_CONFIG_HOME instead. The cost I am
> naming, not glossing over, is that a real GSettings schema gives every
> key a type the compiler enforces, a `gsettings` CLI and dconf-editor
> entry for free, and change notification that works across processes;
> this JSON file gives none of that... I chose it anyway because it needs
> no install step for a checkout to run, which matters more here than
> those properties do."

This is now a legitimate, scoped tradeoff: the *file and its translations*
are no longer dropped (they live in the ported gschema file), and the
remaining substitution (JSON-backed keys vs. a compiled schema) is named
with its actual cost instead of a euphemism. Verified, not taken on faith:
re-read the full comment and the new schema file side by side this pass.

## New finding — a drop with no sentence at all is still the worse failure mode

`lib/config.rb` defines `IS_DEVEL = false`, directly porting upstream's
build-time constant. Upstream gates three real behaviours on it
(`window.rs`'s `devel` CSS class and two status-page icons,
`show_about_dialog`'s version string — see `COMPONENT_PARITY.md` for the
full table). The port's `lib/window.rb` and `lib/about_dialog.rb` implement
all three sites' *unconditional* behaviour and never read
`Config::IS_DEVEL` at all.

This is not an excuse-shaped sentence — there is no sentence. Grepping for
`devel`/`IS_DEVEL` across `lib/window.rb`'s and `lib/about_dialog.rb`'s
comments finds nothing. That is the more serious case by this skill's own
standard: a rationalized drop can at least be read, disagreed with, and
fixed; a silent one survives every review that trusts the file's own
comments, because there is nothing in the file pointing at the gap. Round
1's `component-inventory.md` critique (a "present" ledger read as "present
and correct") applies here too — nothing in any `.reports/` file before
this pass mentioned `IS_DEVEL` at all, so a reader of round 1's reports had
no way to know this gap existed even as an open question.

Severity is modest (a visual ribbon and two icon states, not a workflow),
but it is exactly the shape this skill exists to catch before it compounds:
if the next feature branches on "is this a devel build" anywhere else, it
will branch on a constant that is already proven to go unread.

## A false-green risk checked and ruled out this pass

The runtime gate jumped from 65/14 to 74/15 between rounds. Checked whether
this increase is real: read `test/drive_main_window.rb`'s diff in `e4ebd9f`
directly rather than trusting the number. The +9 checks are the Add PDF
dialog driving (open → bad URL → reject → cancel → still-no-PDF, 6
assertions plus 3 step headers implicitly counted) and they do exercise
real, previously-undriven code (`lib/add_pdf_dialog.rb`'s URL validation
and `lib/entry_page.rb`'s dialog-exposing accessor). This is a genuine
increase in coverage, not a renumbering — confirmed by running the drive
script fresh this pass and getting 74/74 independently, not by reading the
gate's self-report.

## Verdict

Round 1's one real excuse-shaped sentence is fixed at the root (the file it
was excusing not porting is now ported), not just reworded — checked, not
assumed. One new finding: the devel-profile gap carries no excuse at all,
which this skill treats as the more serious case, not a lesser one, because
nothing in the tree flags it for a future reader to find. The runtime
gate's improved numbers were independently re-run and hold up as real
coverage, not a false green.
