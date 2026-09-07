# 02. The action rows, and Open repo

## What to build

Two rows at the foot of the Repo dropdown, and the first of them working.

**One control, not two.** The Repo dropdown is drawn in two places — the compose
page's, before a repo has been picked, and the Repo panel a draft's composer
opens — and they are not the same control today: the first is the app's own
listbox, drawn out of ordinary elements, and the second is the native picker. A
native option that acts rather than picks is precisely the bug class the picking
module was written against, so the panel's picker is redrawn as the listbox the
compose page's already is, and the rows are then added once. Nothing else about
that panel changes: it lists the same repos, sends the same move, and is disabled
in the same two states.

**The listbox gains a row kind.** Rows that press rather than pick, behind a
rule at the foot of the list: never reachable as a choice, never what the closed
control shows, and never mistaken for the choice being gone. The keyboard walks
them with the rest and Enter presses one instead of taking it.

**The two rows are Create repo and Open repo**, in that order, and this task
wires the second. It opens a modal on the existing modal component holding the
registration form's path field — browsing anywhere, repositories marked — with
the same refusals the settings page draws, said under the field with the modal
left open, because a refusal is answered by correcting the path.

**Registration answers with the Repo.** It answers a bare outcome today, and the
modal has to put the draft on what it just registered — the path that was typed
is not the resolved path the Repo is recorded under, so matching it against the
list afterwards is a guess. The outcome that added it carries the Repo, and so
does the one that found it registered already, which is then not a dead end but
a repository to land on. The settings pane reading the same answer goes on saying
exactly what it said.

**And the Repo becomes the draft's**: the compose state's repo id where the page
is composing, a move on a saved draft, exactly as picking a registered one is.

## Acceptance criteria

- [ ] Both places draw the two rows behind a rule; neither can be picked as a
      choice, and the closed control never shows one.
- [ ] Registering an existing repository from the modal closes it and leaves that
      Repo picked — the compose state's id on the compose page, a move on a saved
      draft.
- [ ] A refused path leaves the modal open with the reason under the field, and
      a path already registered lands on the Repo it names.
