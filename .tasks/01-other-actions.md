# 01. Other actions, and a roadmap is continued

## What to build

Turn the compose page's *Adopt a roadmap* dropdown into an **Other actions**
menu: the same shared `Menu`, with one nested level per action. This task
lands the menu and its first level, **Continue a roadmap**, holding exactly
the rows the dropdown draws today; the second level, **Wrap up a pull
request**, arrives with task 02 and is not drawn here.

The menu is drawn whenever the box holds no text and nothing is loaded — the
rule that hid the dropdown while a roadmap was loaded or the box was being
typed in stays, and the rule that hid it while nothing was abandoned goes. A
level with nothing under it is **greyed rather than hidden**: the nested row
gains a disabled state, drawn and read as such, that opens nothing. So the
menu with no roadmap abandoned holds one greyed row.

Every word the human reads about adopting a roadmap says *continue* instead:
the level, the draft's adoption page heading and its press. The code, the
endpoints, the store and the vocabulary keep *adopt*, which is the internal
name for the feature and about to be widened to pull requests.

The nested level should keep what the flat dropdown had: the rows read the
roadmap, the repo, the next stage and where it was found; a press loads the
roadmap and shuts the menu; the open menu's rows survive a Nudge; and the menu
leaves no shadow of itself behind. Move the existing suites to the new shape
rather than writing beside them.

## Acceptance criteria

- [ ] With no roadmap abandoned and an empty box, the menu is drawn and its
      roadmap level reads disabled and opens nothing.
- [ ] With a roadmap abandoned, the level opens to the same rows as before,
      a press loads the roadmap into the composer, and the menu is gone while
      it is loaded or while the box holds text.
- [ ] The adoption page and its press say *continue*, no human-facing string
      says *adopt*, and the web suites pass under the new names.
