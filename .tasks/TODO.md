# The workbench key, and Remote access

A session cannot reach the workbench's API, and a phone can. One long-lived
secret in the Data Directory is what the viewer's own namespace and every page
of the workbench answer 401 without; the link that hands it over is the address
with `?key=…`, and the install's own first-visit path — the daemon's startup
line, the desktop tray's **Open** — is where somebody meets it.

The key is what makes **Remote access** a settings section rather than a recipe
in the adoption docs: a phone cannot reach the workbench until it holds the key,
and `tailscale serve --bg 8422` was a command somebody ran by hand. The section
says whether Tailscale is there and up, turns serve on and off, names the
address, and draws the login link as a QR code with **Reset key** under it. A
dismissable banner on the Conversation page points at it, at the one moment the
human has nothing to do at the desk.

Roadmap stage: [02: The workbench key, and Remote access](docs/roadmaps/onboarding/02-workbench-key-and-remote-access.md)

## Tasks

- [x] 01: The key and the gate — [details](01-key-and-gate.md)
- [x] 02: Handing the link out — [details](02-handing-the-link-out.md)
- [x] 03: Remote access — the pane that reads — [details](03-remote-access-pane.md)
- [x] 04: The serve switch and the operator grant — [details](04-serve-switch-and-operator-grant.md)
- [x] 05: The desktop's graphical sudo — [details](05-desktop-graphical-sudo.md)
- [x] 06: The QR, the copyable link, and Reset key — [details](06-qr-link-and-reset-key.md)
- [x] 07: The banner — [details](07-grilling-banner.md)
- [x] 08: Docs and vocabulary — [details](08-docs-and-vocabulary.md)
