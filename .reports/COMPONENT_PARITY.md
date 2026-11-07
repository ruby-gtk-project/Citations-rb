# Component parity — Citations (audit pass, 2026-11-07, round 2)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `e4ebd9f` |
| Runtime gate this round | `drive_ok=true`, 74/74 checks, 0 failed, 15 components claimed |

Round 1's three gaps (dropped gschema file, undriven Add PDF dialog,
GObject-CRITICAL on `EntryList#filter`) are re-verified fixed — see
`COMPONENT_IDENTIFICATION.md` for the evidence on each. This round found one
new gap and re-ran the runtime gate independently (`nix develop --command
xvfb-run -a ruby test/drive_main_window.rb`, confirmed 74/74, `DRIVE OK`, no
stderr CRITICAL lines).

## Gap — the devel build profile has no visible or behavioural effect

| Axis | Upstream (`IS_DEVEL`) | Port (`Config::IS_DEVEL`) | Verdict |
|---|---|---|---|
| Window CSS | `devel` class added to the `AdwApplicationWindow` in `constructed()` | never added, under any condition — `grep -rn 'devel' lib/window.rb` is empty | **gap** |
| Empty-state icons | `recent_files_status_page`/`no_recent_files_status_page` get the app-id icon **only** when devel | both status pages get `Config::APP_ID` as their icon **unconditionally** | **gap** (inverted: port behaves as "always devel" for this one icon) |
| About dialog version | `dialog.set_version(VERSION)` **only** when devel; otherwise the version shown comes from the metainfo `<releases>` list via `from_appdata` | `about.version = @version` **unconditionally**, every build | **gap** |

`Config::IS_DEVEL` is defined (`lib/config.rb:18`, hardcoded `false`) and
never read anywhere else in `lib/` or `exe/`. The constant exists; the
behaviour it should gate was never wired to it. None of the three sites
carry a comment naming this as a known or deliberate omission — it is not
an accepted correspondence, it is a drop with no record.

Impact is real but narrow: a developer/nightly build of the port would be
visually indistinguishable from a stable one (no devel ribbon), and the
About dialog always shows the raw `VERSION` string rather than following
upstream's devel/stable split. Low severity on its own, but it is the kind
of gap the runtime gate cannot see — no driver check asserts on CSS classes
or on `Config::IS_DEVEL` toggling, so it will stay invisible to every future
"74/74 passed" claim until someone reads `window.rs` side by side with
`window.rb` again.

## What held up this round

- All 8 `gschema` keys, defaults, and `<summary>` strings — byte-identical,
  file present at `data/org.rubygtkproject.Citations.gschema.xml.in`.
- Add PDF dialog — driven end to end (open empty → type bad URL → reject
  without network call, error style applied → cancel → entry still has no
  PDF). Note: the *successful*-download path (a real/good PDF URL) is still
  not exercised by any check — not a component gap (the dialog's happy path
  exists and is identical to upstream's `reqwest`-based `DownloadManager`
  call pattern on inspection), but worth naming so it isn't mistaken for
  fully covered.
- `EntryList#filter` — no CRITICAL on any of the two fresh drive runs this
  pass; the `nil`-expression-then-setter workaround is silent and the
  resulting `Gtk::StringFilter` behaves correctly per driver checks 6–9
  (search narrows the list, clearing restores it).
- Full action/accelerator/shortcut inventory (see
  `COMPONENT_IDENTIFICATION.md`) — complete, 1:1.
- The `Gtk.SourceView` → `Gtk.TextView` downgrade (no BibTeX/LaTeX syntax
  highlighting in the snippet editor or the rendered-citation preview) is a
  real, standing functional gap versus upstream, but it is the one gap in
  this entire audit that is honestly and specifically documented in both
  affected files' class comments (`lib/new_entry_dialog.rb`,
  `lib/citation_preview.rb`) — named, not hidden, so it is recorded here as
  a known limitation rather than a fresh finding.

## Verdict

Not clean. Round 1's three gaps are fixed. One new gap found this round:
the devel-profile distinction (CSS class, two status-page icons, About
dialog version) is dropped with no record anywhere. The runtime gate's
74/74 does not and cannot exercise this, because nothing in the drive
script builds the app with `IS_DEVEL` true or asserts on CSS classes.
