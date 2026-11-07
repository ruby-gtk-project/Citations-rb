# Test parity — Citations (audit pass, 2026-11-07, round 3)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `d989da7` |
| Upstream `#[test]` cases | 22 |
| Named by a port test | 29 `T.test(...)` blocks, a superset covering all 22 |
| Test commands run this pass, fresh | `nix develop --command xvfb-run -a ruby test/test_models.rb` → 93/93; `nix develop --command xvfb-run -a ruby test/test_texer.rb` → 84/84; `nix develop --command xvfb-run -a ruby test/drive_main_window.rb` → 78/78, `DRIVE OK`, no stderr CRITICAL |

This suite is not "ported but never executed" — it was run, in full, by
this pass, independently of any prior report's claimed numbers.

## Upstream test census, counted directly from `origin/master`

```
$ for f in cratebibtex/src/{entry,format,lib}.rs src/utils.rs texer/src/tests.rs; do
    git show origin/master:"$f" | grep -c '#\[test\]'
  done
1   # entry.rs
4   # format.rs
1   # lib.rs
2   # src/utils.rs
14  # texer/src/tests.rs
```

22 total. `test/test_models.rb` carries 15 `T.test(...)` blocks and
`test/test_texer.rb` carries 14, 29 combined. The extra 7 beyond upstream's
22 are added coverage, named and matched by reading, not assumed from the
count alone:

| Port test | Maps to |
|---|---|
| `format::test_clear_citation`, `format::test_format_authors`, `format::test_clear_latex`, `format::test_clear_markdown` | `format.rs`'s 4 |
| `cratebibtex::test` | `lib.rs`'s 1 |
| `entry::format`, `entry::serialize` | `entry.rs`'s 1 (split into two assertions) |
| `utils::test_generate_key`, `utils::test_sanitize_key` | `src/utils.rs`'s 2 |
| `bib::list_model`, `bib::round_trips_through_a_file`, `entry_type::fields_and_names`, `format::kinds`, `utils::urls_and_filenames`, `settings::defaults_match_the_upstream_schema` | added, no upstream `#[test]` counterpart — extra coverage, not a gap |
| `test/test_texer.rb`'s 14 names | 1:1 against `texer/src/tests.rs`'s 14, by name (`parse_method_name`, `parse_method_with_arg`, `parse_method_without_arg`, `parse_method`, `parser`, `test_35`, `test_46`, `delimited_by_brackets`, `texer`, `multispacer`, `till_symbols_no_ws_once`, `till_symbols_no_ws`, `tag_consecutive_spaces`, `test_59`) |

All 22 upstream cases are represented. 0 are missing.

## Drive suite

78 checks, 14 components per this round's gate — down from round 2's
reported 74/15. This pass did not chase that exact number against any file
(no `components.json` or similar is tracked in the repo; "components" is
an external accounting this report cannot independently recompute), but it
did re-verify the two things that number could be hiding:

- **No regression in what is driven.** Every component round 2's drive
  script exercised is still exercised: recent files, the sidebar, the
  editor (extras, preview, PDF row), the Add PDF dialog's reject path, the
  custom template editor, opening a second window from argv.
- **More is driven, not less, versus round 2's 74.** The Add PDF dialog's
  successful-download path (checks 39-42) is new since round 2 and is a
  real gain in coverage: a loopback HTTP server, a real one-page PDF
  built with `Cairo::PDFSurface`, the download completing, the PDF
  landing on disk, and the preview rendering a real `Gdk::Texture`.

The devel-profile branch (`Config::IS_DEVEL` true) is not driven by any
check on either round's suite — see `COMPONENT_PARITY.md`. That is a real
coverage gap in the gate: the three sites behave correctly on inspection,
but no check ever flips the flag and asserts on the CSS class, the icon, or
the About dialog version actually changing. Noted here because "255/255"
is otherwise read as "nothing is unverified," and that one branch is.

## Verdict

Clean on the question this skill asks directly: every upstream test is
ported, named, and the suite was actually executed this pass (not merely
claimed) — 255/255 across all three entry points, 0 failures, re-run
independently rather than trusted from a prior report. The one standing gap
is coverage of the devel-profile branch, which is a gate-coverage note
rather than a test-parity failure (the behaviour it would exercise already
has component parity per `COMPONENT_PARITY.md`).
