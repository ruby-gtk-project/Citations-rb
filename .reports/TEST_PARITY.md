# Test parity — Citations (audit pass, 2026-11-07)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `b9516f9` |
| Upstream `#[test]` cases | 22 |
| Named by a port test | 22 (by name, see census below) |
| Test commands run this pass | `nix develop --command xvfb-run -a ruby test/test_models.rb` (93/93), `nix develop --command xvfb-run -a ruby test/test_texer.rb` (84/84), `nix develop --command xvfb-run -a ruby test/drive_main_window.rb` (65/65), `ruby -e 'require "rspec/core"; ...'` (**fails to load**) |

## Census — upstream `#[test]` functions

| File | Count | Names |
|---|---:|---|
| `cratebibtex/src/entry.rs` | 1 | `format` |
| `cratebibtex/src/format.rs` | 4 | `test_clear_citation`, `test_format_authors`, `test_clear_latex`, `test_clear_markdown` |
| `cratebibtex/src/lib.rs` | 1 | `test` |
| `src/utils.rs` | 2 | `test_generate_key`, `test_sanitize_key` |
| `texer/src/tests.rs` | 14 | `test_parse_method_name`, `test_parse_method_with_arg`, `test_parse_method_without_arg`, `test_parse_method`, `test_parser`, `test_35`, `test_46`, `test_delimited_by_brackets`, `test_texer`, `test_multispacer`, `test_till_symbols_no_ws_once`, `test_till_symbols_no_ws`, `test_tag_consecutive_spaces`, `test_59` |

All 22 names have a same-named counterpart in `test/test_models.rb` (format,
entry, bib, utils cases) or `test/test_texer.rb` (the 14 texer cases).
Each was spot-read, not just name-matched: `test_generate_key` and
`test_sanitize_key` in `test/test_models.rb` assert the same input/output
pairs as `src/utils.rs`'s versions. **This bijection is real**, not a
name-matching coincidence, for the cases spot-checked; a full line-by-line
re-verification of all 22 was not repeated in this pass since prior work
appears to have already done this (`test/test_models.rb`'s own header names
each test after its upstream counterpart deliberately).

The port's suites also carry extra cases with no upstream counterpart
(`entry::serialize`, `bib::list_model`, `bib::round_trips_through_a_file`,
`entry_type::fields_and_names`, `format::kinds`, `utils::urls_and_filenames`,
`settings::defaults_match_the_upstream_schema`) — correctly listed as extra,
not counted toward the 22.

## Gap — the suite the dev shell does not mention

The dev-shell banner prints exactly three commands:

```
run the app:   xvfb-run ruby exe/citations
drive the app: xvfb-run ruby test/drive_*.rb
run the specs: ruby -e 'require "rspec/core"; exit RSpec::Core::Runner.run(...)'
```

It never mentions `test/test_models.rb` or `test/test_texer.rb` — the two
files that carry all 22 ported upstream tests. Running `ruby test/test_models.rb`
without `xvfb-run` (i.e. following only the banner's advertised pattern for
anything that isn't `drive_*`) fails outright:

```
Gtk::Loader#call_init_function: failed to initialize GTK (Gtk::InitError)
```

because `lib/bib.rb` requires `gtk4` unconditionally even for model-only
tests. Both model suites do pass once run under `xvfb-run` (confirmed this
pass: 93/93 and 84/84), so the tests themselves are not broken — but a
newcomer following the dev shell's own banner would never discover they
need `xvfb-run` for a "non-widget" test file, and the banner's omission of
these two files at all means a reviewer who only runs what the shell
advertises never executes 22/22 of the ported upstream tests.

## Gap — the RSpec harness does not run, at all

```
$ nix develop --command ruby -e 'require "rspec/core"; exit RSpec::Core::Runner.run(["--format","progress","--require","spec_helper","spec"])'
LoadError: cannot load such file -- kettle/test/rspec
	# ./spec/spec_helper.rb:23
0 examples, 0 failures, 2 errors occurred outside of examples
```

`spec/` contains only `spec_helper.rb` and `README.md` — **no `*_spec.rb`
file exists anywhere in the repository**, upstream or port. Even if the
`kettle/test/rspec` dependency were present, this harness would report
"0 examples, 0 failures" and look green. As shipped it does not even load.
This is a suite that has never been executed and never could produce a
real result — the exact false-green shape this round's instructions call
out explicitly. It is dead scaffolding (from the `kettle-jem` packaging
commits), not a test suite, and the third line of the dev-shell banner
advertises it as one.

## Verdict

22/22 upstream tests are named and pass under the correct invocation
(93+84=177 checks, `xvfb-run` required and undocumented in the banner for
two of the three suites). The drive suite is 65/65 but — per
`COMPONENT_PARITY.md` — does not reach `lib/add_pdf_dialog.rb` at all, so
"14 components" covered by the drive gate excludes at least one real
component file. The RSpec suite is advertised, cannot load, and has zero
spec files. Not clean.
