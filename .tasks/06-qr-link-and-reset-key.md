# 06. The QR, the copyable link, and Reset key

## What to build

The rest of the Remote access pane: the login link made usable from a phone, and
the press that takes it back.

**The QR code is drawn client-side**, as inline SVG from a small encoder — a
dependency the viewer takes rather than something written from scratch. Nothing
is fetched to draw it: a workbench standing behind a key has no business asking
a third party to render that key, and an install on a tailnet may have nowhere
to fetch from anyway.

**The link itself sits beside it, copyable**, for when a camera is the wrong
tool — a laptop on the same tailnet, a link pasted into a note.

**Reset key sits under them.** It re-issues the secret, which logs every device
out: the QR and the link redraw on the new key, and whatever was holding the old
one meets a 401 on its next request. That is the whole of what makes a lost
phone recoverable, and it is also how somebody signs out the workbench they are
standing in — so it reads as a press with consequences rather than a tidy-up.

## Acceptance criteria

- [x] The pane draws a QR of the login link as inline SVG, fetching nothing, and
      a phone reading it lands in a logged-in workbench.
- [x] The link beside it copies.
- [x] **Reset key** re-issues the secret, and the QR and the link redraw on it.
- [x] A device holding the previous key gets 401 on its next request.
