# Component parity — Citations (audit pass, 2026-11-07)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `b9516f9` |
| Runtime gate this round | `drive_ok=true`, 65/65 checks, 0 failed, 14 components claimed |

No prior `COMPONENT_PARITY.md` existed on this branch; this is the first.
Scope: the gaps found this pass, not a re-derivation of the whole 14-unit
survey already in `.reports/component-inventory.md` (which this audit spot-
checked and found accurate as a static inventory).

## Gap 1 — `data/org.gnome.World.Citations.gschema.xml.in` has no counterpart

| Axis | Upstream | Port | Verdict |
|---|---|---|---|
| Count | 1 schema file, 8 keys, 7 distinct translatable summaries | 0 files | **gap** |
| Style | n/a | n/a | — |
| Behaviour | `gio::Settings` + change notification | `lib/settings.rb`'s JSON file + `connect_changed` reproduces the *keys and notification*, correctly | correspondence (see below) |
| Text | 7 `<summary>` strings | 0 | **gap** — tracked in `TRANSLATION_PARITY.md` |

The *behaviour* (8 keys, same defaults, same change-notify semantics) is a
legitimate correspondence — `lib/settings.rb`'s header comment explains the
substitution and names the reason (no schema-install step needed). That part
is fine per `component-parity`'s own rules on correspondences.

What is not a correspondence: the **file and its 7 translatable summaries**
have no port counterpart anywhere, by any name. `lib/settings.rb`'s own
`DEFAULTS` hash carries the 8 keys with no translation marker on any of
them, and no other file mentions the summary text. This is a straight
component gap (missing file, missing text) riding inside what reads, on
first glance, as an accepted substitution — see `ACCOUNTABILITY_ENSURANCE.md`
for why that framing is itself the problem.

## Gap 2 — `lib/add_pdf_dialog.rb` is unverified

| Axis | Upstream | Port | Verdict |
|---|---|---|---|
| Count | `AddPdfDialog` (`src/add_pdf_dialog.rs` + `add_pdf_dialog.ui`), 16 widgets | `lib/add_pdf_dialog.rb`, present, same widget counts per the static inventory | present (static) |
| Behaviour | opens from the PDF stack, accepts a file or a URL, validates, writes `pdf-path`/`pdf-url` | **never driven** — `test/drive_main_window.rb` has no reference to it at all | **unverified** |

The runtime gate's 65/65 says nothing about this component: it is not in the
denominator. Per this round's instruction, a component no check exercises is
a gap in its own right, not a pass by omission. Opening the dialog, typing a
URL, cancelling, and asserting the resulting entry field are all untested.

## Gap 3 — `EntryList#filter` throws a GObject-CRITICAL on every build

```
Gtk::StringFilter.new(Gtk::PropertyExpression.new(Entry, nil, 'search-text'))
```

`nil` in the second argument position produces
`g_object_ref: assertion 'G_IS_OBJECT (object)' failed` from GObject-
Introspection on every `Window.new` — reproduced twice via
`nix develop --command xvfb-run -a ruby test/drive_main_window.rb`. None of
the 65 checks assert on stderr, so this passes the gate while being a live
defect in a shipped component (the sidebar search filter). Whether the
filter functions correctly despite the warning was not independently proven
here — the warning itself is reason enough to record this as a behaviour
gap pending investigation, not a cosmetic nit.

## What held up

Spot-checked and found correct this pass: the 6 built-in citation templates
(`Mla`/`Ams`/`Apa`/`Chicago`/`Harvard`/`Vancouver`) plus `Custom` in
`lib/citation_preview.rb`, matched against `src/citation_preview.rs`'s
`Template` enum and its templates, including the exact (byte-identical once
XML entities are decoded) custom-template help string; the `po/` locale set
(43 files both sides, `LINGUAS` matches exactly); the accelerator table in
`lib/main.rb` against `src/application.rs`.

## Verdict

Not clean. 2 confirmed component gaps (schema file + its text, the
unverified Add-PDF dialog) and 1 confirmed runtime defect (GObject-CRITICAL
on `EntryList#filter`), none of which the 65/65 runtime gate catches.
