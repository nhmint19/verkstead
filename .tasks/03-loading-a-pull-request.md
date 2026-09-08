# 03. Loading a pull request into the composer

## What to build

Picking a free row loads the pull request into what the compose page holds,
the way picking a roadmap does, and both presses create a Draft that records
it.

Loaded, the composer draws a card over the box naming the pull request — repo,
number, title, head branch and the base it goes into — with a clear control.
The box stays a box: it is prefilled with the title as a heading and the
description under it, editable markdown, and what is left there is the Brief.
Text that was in the box is put away and given back on clear, as a held file
is. The Repo is the pull request's and its picker reads disabled; the branch
field, the base picker and the grilling Pairing picker are not drawn; the
Implementation and Review pickers are, on the Repo's remembered prefill; the
companions are the human's. The loaded state is kept on the device and
survives a reload.

**Save as draft** creates a Conversation in Draft that records which pull
request it holds — number, title, URL, head branch and base branch, alongside
the roadmap adoption record's shape rather than in it — saves the edited Brief,
and replays the touched pickers and companions. **Start work** does the same
and then presses the take-up, which task 04 builds; until then it stops where
Save as draft stops. Readiness for the press is the two roles picked, the Brief
having arrived with the pull request.

The Draft's own page draws what the compose page drew: the card over the
editable Brief, the two pickers, the companions and one press. Its Repo picker
refuses a move, as an adopting Draft's does, and says why in the same place.

## Acceptance criteria

- [ ] Loading a pull request over typed text prefills the box with the title
      and description, survives a reload, and clear restores what was typed.
- [ ] Save as draft creates a Conversation whose page names the pull request,
      holds the edited Brief, draws the two pickers and no branch, base or
      grilling picker.
- [ ] A Draft holding a pull request refuses a Repo move by name.
