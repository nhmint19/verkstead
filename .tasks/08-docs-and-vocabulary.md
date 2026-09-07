# 08. Docs and vocabulary

## What to build

The words for what the seven tasks before this one built.

**CONTEXT.md gains two terms** in the workbench group: **Workbench Key** — the
one long-lived secret a browser holds and a session cannot, what it gates and
what stays open beside it, and that resetting it logs every device out — and
**Remote Access**, the settings section that turns `tailscale serve` on and
hands the key to a phone. Written the way the terms around them are: what it is,
what it is not, and an _Avoid_ line where there are words to steer off.

**`adoption.md` tells the first-visit story per install**, where each install is
described rather than gathered into a section of its own: the daemon's startup
line on NixOS, and the tray's **Open** on each of the three desktop apps. Its
`tailscale serve --bg 8422` recipe stops being a command somebody runs by hand
and becomes a pointer at the pane, and the same recipe in `development.md` goes
the same way. The module's own option documentation says what the operator grant
is for and when the module sets it.

## Acceptance criteria

- [ ] CONTEXT.md carries **Workbench Key** and **Remote Access**, written as the
      terms around them are.
- [ ] `adoption.md` says how the link is found on each install it describes.
- [ ] The `tailscale serve` recipes in `adoption.md` and `development.md` point
      at the pane instead of being run by hand.
- [ ] Nothing in the documentation still tells somebody to run a command the
      pane now runs for them.
