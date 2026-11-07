# Translation parity — Citations (audit pass, 2026-11-07, round 2)

| | |
|---|---|
| Upstream | `origin/master` |
| Port | `ruby` @ `e4ebd9f` |
| Upstream `.po` files | 43, matching `LINGUAS` exactly |
| Port `.po` files | 43 — basename diff empty (unchanged from round 1) |

## Round 1's 7-message gap, re-verified fixed

Round 1 found 7 msgids with no port counterpart anywhere, all from
`data/org.gnome.World.Citations.gschema.xml.in`, which had no port file at
all. `e4ebd9f` ported the file to
`data/org.rubygtkproject.Citations.gschema.xml.in`:

```
$ diff <(grep -oP '<summary>\K[^<]+' /tmp/.../upstream/data/org.gnome.World.Citations.gschema.xml.in) \
       <(grep -oP '<summary>\K[^<]+' data/org.rubygtkproject.Citations.gschema.xml.in)
(empty)
```

All 7 strings — `Window width`, `Window height`, `Window maximized state`,
`List of paths of recent files`, `Default template for book citation`
(appearing twice, upstream's own duplicate kept as-is and noted in the
port file's comment rather than silently "fixed"), `Default template kind
for citations`, `Default template format for citations` — are
byte-identical in the new file. `po/POTFILES.in` was updated to include it
and `po/citations.pot` carries all 7 msgids (confirmed: `grep -n "Window
width" po/citations.pot` finds the new entry at line 670). These are new
msgids as of this commit, so no `.po` file has a translated string for them
yet (confirmed empty on `po/fr.po`) — that is the same state upstream's own
untranslated-until-picked-up strings are in after any schema change, not a
port defect.

## Flagged-but-functional risk, re-confirmed present and unchanged

Round 1 flagged `lib/citation_preview.rb`'s `TEMPLATES` table: the 7
template-kind labels (`Mla`, `Ams`, `Apa`, `Chicago`, `Harvard`,
`Vancouver`, `Custom`) are passed to `_(name)` as a variable, not a string
literal, so a real `xgettext`/census tool run against this file alone would
never discover them as translatable. This is unchanged by `e4ebd9f` (that
commit did not touch `citation_preview.rb`'s template table) and remains
true: the app translates correctly at runtime today because the `.po`
files already carry these 7 keys from upstream's own `gettext("Chicago")`
etc. calls, carried forward verbatim. The risk is only forward-looking (a
future from-scratch `.pot` regeneration from this file alone would silently
drop these 7), so it is recorded again here, not re-flagged as a new
finding.

## New this pass — no new gap found

This pass re-ran the ad hoc msgid census from round 1 against the new
gschema file and against every `.rb` file touched by `e4ebd9f`
(`lib/entry_list.rb`, `lib/entry_page.rb`, `lib/settings.rb`). None of
those three introduce new user-facing strings (the settings class comment
is a source comment, not a translatable string; the entry_list/entry_page
changes are binding-usage fixes and a new accessor method, not new UI
text). No new gap found.

## Held up, unchanged from round 1

Locale set (43/43), the custom-template help string (byte-identical once
`.ui` XML entities are decoded), and the `shortcuts.ui` ↔
`shortcuts_dialog.rb` `p_('shortcut window', …)` context strings all still
hold up — re-spot-checked this pass, no regression.

## Verdict

Not fully clean, but meaningfully improved: round 1's confirmed 7-message
gap is fixed and verified byte-identical. The one standing risk (the
`_(name)` variable-argument template labels, functionally fine today,
fragile for a future `.pot` regeneration from this file) is unchanged and
recorded again for the same reason as before — a real risk, not a crisis,
and it is not invisible: `TRANSLATION_PARITY.md` itself is now where it is
tracked. No new translation gap found this pass.
