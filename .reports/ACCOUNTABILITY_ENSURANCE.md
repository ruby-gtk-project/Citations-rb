# Accountability ensurance — Citations (audit pass, 2026-11-07)

Scanned `lib/`, `exe/`, `.reports/`, and the repo root for `TODO`, `FIXME`,
`not implemented`, `for now`, `in a future`, `dropped deliberately`,
`not applicable`, `n/a`, `out of scope`, `by design`, `simplified`, and for
any claim of completeness resting on a check that was never run.

## No labelled skips found

`grep -rniE 'TODO|FIXME|not implemented|for now|in a future'` over `lib/`
and `exe/` returns nothing but one identifier (`SKIPPED_BLOCKS` in
`lib/bib.rb`, a real constant naming BibTeX `@preamble`/`@comment` blocks
that nom-bibtex also discards — not an excuse, a correct port of upstream's
own behaviour). No `PORTING.md`, `FINDINGS.md`, `COMPONENT_PARITY.md`,
`TEST_PARITY.md` or `TRANSLATION_PARITY.md` existed on this branch before
this pass — only `.reports/component-inventory.md` — so there was no prose
ledger to mine for excuse language, which also means there was no ledger at
all tracking the two real gaps found this pass.

## One sentence that is doing exactly what this skill exists to catch

`lib/settings.rb`, in its class comment:

> "Upstream reads them through gio::Settings, which needs a compiled schema
> installed into the system's glib-2.0/schemas directory; requiring an
> install step before the app can run from a checkout is worse than the
> alternative, so the same keys are backed by a small JSON file..."

This reads as architecture. It is not. Under the rule this skill states,
there is no valid reason to drop a file upstream ships — "an install step
would be worse" is exactly the "by design" construction the skill lists by
name, wearing a technical justification instead of the word "design". It
reads calmly, which is the failure: a calm sentence is a sentence a future
agent can cite as settled, and in fact nobody since has gone back to check
what that substitution actually cost. It cost the schema file and its 7
translatable summary strings, with no counterpart anywhere in the port
(`COMPONENT_IDENTIFICATION.md`, `TRANSLATION_PARITY.md`). Rewritten per the
skill's rule:

> I did not port `data/org.gnome.World.Citations.gschema.xml.in` or its 7
> translatable summary strings, because I am an idiot who decided a feature
> I didn't understand wasn't needed. The JSON-file substitution for the
> *keys* was a legitimate call; I used it to also justify dropping the
> *file* and its translations, which nobody asked me to drop and which no
> binding limitation required.
> Upstream: `data/org.gnome.World.Citations.gschema.xml.in`, 8 keys, 7
> distinct `<summary>` strings.
> Port: `lib/settings.rb`, same 8 keys and defaults, zero translation
> markers, no schema file anywhere in the tree.

## A green claim resting on something never executed

`.reports/component-inventory.md` states flatly, for every one of 14 files,
that the column that mattered "is now present everywhere". That sentence is
true of file existence and was read, before this pass, as if it also meant
"behaves correctly" — it does not say that, but nothing in the repo ever
checked it, and the gap between the two readings is exactly where
`lib/add_pdf_dialog.rb` (present, 330 lines, zero driver coverage) and the
`EntryList#filter` GObject-CRITICAL (present, passes every check, throws on
every window build) were sitting unexamined. Neither is excused by any
sentence anywhere — nobody wrote "Add PDF doesn't need testing" or "the
filter warning is harmless". The risk here is narrower and still real: a
document that says "present" next to 14 files, re-read six commits later by
someone in a hurry, is read as "present and verified" even though it never
claimed the second half. That gap in what the document implies versus what
it checked is itself worth naming, because it is exactly the mechanism this
skill exists to interrupt before a false "present" calcifies into a false
"done".

The RSpec harness (`spec/spec_helper.rb` requiring `kettle/test/rspec`,
which is not installed, over zero `*_spec.rb` files) is advertised by the
dev shell's own banner as "run the specs" with no caveat. Nobody wrote a
sentence excusing it; it simply was never run and nothing in the repo says
so. Recording it here because a banner that advertises a broken, empty
command is a false claim of readiness even without a single excuse-word
anywhere near it.

## Verdict

One real excuse-shaped sentence, rewritten above, sitting on top of a real
7-message translation gap and a real missing file. Two further gaps
(`add_pdf_dialog.rb` undriven, the `EntryList#filter` CRITICAL) that nobody
excused in prose but that a "present" ledger let stand unexamined. Not
clean — and the fix for the sentence is not a better sentence, it is
porting `data/org.gnome.World.Citations.gschema.xml.in`.
