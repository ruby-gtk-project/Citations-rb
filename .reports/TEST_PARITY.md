# Test parity — Citations (audit pass, 2026-11-08, round 4)

| | |
|---|---|
| Upstream | `origin/master` @ `e360482` |
| Port | `ruby` @ `265669c` |
| Upstream `#[test]` cases | 22 |
| Named by a port test | 29 `T.test(...)` blocks, a superset covering all 22 |
| Test commands run this pass, fresh | `nix develop --command xvfb-run -a ruby test/test_models.rb` → 93/93; `nix develop --command xvfb-run -a ruby test/test_texer.rb` → 84/84; `nix develop --command xvfb-run -a ruby test/drive_main_window.rb` → 78/78, `DRIVE OK`; `nix develop --command xvfb-run -a ruby test/drive_devel_build.rb` → 4/4, `DRIVE OK` (new this round) |

Every entry point in `test/` was run fresh this pass, not trusted from a
prior report's numbers.

## Upstream test census, re-counted directly from `origin/master` at this round's sha

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

22 total, unchanged from round 3 (`origin/master` moved from `d989da7`'s
upstream reference by zero commits relevant to these five files — the sha
recorded here, `e360482`, is current `origin/master`).

`test/test_models.rb` carries 15 `T.test(...)` blocks and `test/test_texer.rb`
carries 14, 29 combined, re-counted this pass:

```
$ grep -c "T.test" test/test_models.rb test/test_texer.rb
test/test_models.rb:15
test/test_texer.rb:14
```

| Port test | Maps to |
|---|---|
| `format::test_clear_citation`, `format::test_format_authors`, `format::test_clear_latex`, `format::test_clear_markdown` | `format.rs`'s 4 |
| `cratebibtex::test` | `lib.rs`'s 1 |
| `entry::format`, `entry::serialize` | `entry.rs`'s 1 (split into two assertions) |
| `utils::test_generate_key`, `utils::test_sanitize_key` | `src/utils.rs`'s 2 |
| `bib::list_model`, `bib::round_trips_through_a_file`, `entry_type::fields_and_names`, `format::kinds`, `utils::urls_and_filenames`, `settings::defaults_match_the_upstream_schema` | added, no upstream `#[test]` counterpart — extra coverage, not a gap |
| `test/test_texer.rb`'s 14 names | 1:1 against `texer/src/tests.rs`'s 14, by name |

All 22 upstream cases are represented. 0 are missing. This is unchanged from
round 3 — `265669c` touched no test file.

## Drive suite

78 checks against `test/drive_main_window.rb`, matching this round's stated
gate exactly, re-run fresh. **New this round:** `test/drive_devel_build.rb`,
a fourth entry point added in `265669c`, 4 checks, driving the
`Config::IS_DEVEL` branch end to end (the window's `devel` CSS class, both
status pages' app-id icon, the About dialog's raw `VERSION` string) — see
`COMPONENT_PARITY.md`. This closes the one coverage gap round 3 flagged:
"no driver check builds the app with `IS_DEVEL` true." It now does, in a
script of its own rather than inside the 78.

- **No regression in what is driven.** Every component round 3's drive
  script exercised — recent files, the sidebar, the editor, the Add PDF
  dialog's both branches, the custom template editor, the second-window-from-argv
  path — is still exercised, unchanged, in the same 78 checks.
- **Total runtime checks across all four entry points this pass: 78 + 4 +
  93 + 84 = 259, all green, zero stderr `CRITICAL`.** The task's stated gate
  (78/18 components) refers to `drive_main_window.rb` alone; the 259 figure
  is the honest total across everything the repo actually runs.

## Verdict

Clean on the question this skill asks directly: every upstream test is
ported, named, and every suite in the repo was actually executed this pass
— 259/259 across all four entry points, 0 failures, re-run independently.
The devel-profile branch, which round 3 named as the one standing coverage
gap in the *gate* (not a test-parity gap, since the behaviour it exercises
already had component parity), is now driven by its own script. No
remaining coverage gap in the runtime gate identified this pass.
