# 04. The Create repo modal

## What to build

The other row, and the form behind it.

**Two fields on the existing modal**: a parent directory and a name. The parent
is the app's browsing path field, browsing anywhere — the same field the Open
modal beside it stands on, without repositories marked, because what is being
picked here is where a repository will go rather than one that is already there.

**It remembers the last parent on the device.** Somebody making a second
repository is almost certainly putting it beside the first, and where they keep
their code is a fact about the machine in front of them rather than something to
tell the server. Kept in the browser storage the app's other per-device settings
live in; where there is none — which on a first run there never is — the browse
opens at the server's own home, which is where an unbounded browse already opens.

**The refusals are drawn under the field and the modal stays open**, for the
reason the registration form's are: what answers a refusal is correcting what was
typed, and a modal that closed on one would take the correction away with it.

**And the created Repo becomes the draft's**, exactly as the registered one does
in the task before this — the compose state's repo id, or a move on a saved
draft.

The tick that also makes it on GitHub is the task after this one. Nothing here
draws it.

## Acceptance criteria

- [ ] Creating from the compose page closes the modal and leaves the new Repo
      picked, with the draft on it.
- [ ] The parent opens at the last parent this device used, and at the server's
      home where it has none; nothing about that memory reaches the server.
- [ ] A refused create leaves the modal open with the reason under the field and
      nothing picked.
