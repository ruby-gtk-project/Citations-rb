# Test parity — Citations (audit pass, 2026-11-07, round 2)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `e4ebd9f` |
| Upstream `#[test]` cases | 22 |
| Named by a port test | 22/22 (unchanged from round 1's census, re-confirmed) |
| Test commands run this pass | `nix develop --command xvfb-run -a ruby test/test_models.rb` → 93/93; `nix develop --command xvfb-run -a ruby test/test_texer.rb` → 84/84; `nix develop --command xvfb-run -a ruby test/drive_main_window.rb` → 74/74, `DRIVE OK`, no stderr CRITICAL lines |

## Round 1's gap, re-verified fixed

Round 1 flagged two problems: the dev-shell banner never mentioned
`test/test_models.rb`/`test/test_texer.rb` (the files carrying all 22
ported upstream tests) and never said they need `xvfb-run`; and the banner
advertised a dead RSpec harness (`spec/spec_helper.rb` requiring
`kettle/test/rspec`, zero `*_spec.rb` files anywhere) that could not even
load.

Both are fixed in `e4ebd9f`:

```
$ nix develop --command xvfb-run -a ruby test/test_models.rb
Citations devshell — ruby 3.4.9
run the app:    xvfb-run ruby exe/citations
drive the app:  xvfb-run ruby test/drive_*.rb
unit tests:     xvfb-run ruby test/test_models.rb && xvfb-run ruby test/test_texer.rb
                (xvfb-run is needed here too: lib/bib.rb requires gtk4 unconditionally)
93 of 93 checks passed in models
models OK
```

`spec/` no longer exists (`ls spec/` → "No such file or directory"); the
banner's third line now names the real unit-test entry points and the
`xvfb-run` requirement explicitly, rather than an RSpec command that never
worked. This pass re-ran both unit suites fresh and both still pass in
full (93/93, 84/84) — the 22-test bijection against upstream's `#[test]`
functions (`cratebibtex`'s `entry.rs`/`format.rs`/`lib.rs`, `src/utils.rs`,
`texer/src/tests.rs`) that round 1 spot-checked still holds; nothing in
`e4ebd9f` touched the test files' content, only the banner and the dead
`spec/` scaffolding.

One minor, cosmetic leftover: `.rspec` still exists and still configures
`--format html --out results/...` / `--format RspecJunitFormatter`
formatters for a runner the repo no longer advertises or can invoke
(no `rspec` gem is required anywhere, no spec files exist). It is dead
config, not a false claim — nothing in the dev shell, `README`, or banner
points at it — so it is noted here as residue, not scored as a gap.

## Drive suite coverage, re-counted

74 checks, 15 components per this round's gate (up from 65/14 in round 1
— the +9 checks and +1 component are the Add PDF dialog driving added in
`e4ebd9f`, confirmed by reading `test/drive_main_window.rb` checks 33–38 and
counting: 6 new assertions plus the "opens from the empty PDF row" and
"the add pdf dialog opens" step headers). This closes round 1's
"`lib/add_pdf_dialog.rb` is unverified" finding specifically — it is now
inside the denominator, not outside it.

No new undriven component was found this pass that wasn't already a known,
documented limitation (see `COMPONENT_PARITY.md`'s note on the Add PDF
dialog's happy-download path still being unexercised — a narrower,
secondary gap inside an otherwise-driven component, not an unverified
component in the round-1 sense).

## Verdict

22/22 upstream unit tests present, named, and passing under the correct
(now correctly documented) invocation. The drive suite is 74/74 and now
includes the one component round 1 found completely unverified. The dead
RSpec scaffolding and its false banner line are gone. Not fully clean only
because of the cosmetic `.rspec` leftover and the Add PDF dialog's
unexercised success path noted above — neither is a suite that fails to
run, so this report is otherwise clean on the question this skill asks.
