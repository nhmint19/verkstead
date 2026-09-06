# 02. The workbench key, and Remote access

## Goal

A session cannot reach the workbench's API, and a phone can. Demonstrable:
`curl http://127.0.0.1:8422/api/ui/repos` from inside a sandbox answers 401;
the desktop tray's **Open** lands in a logged-in workbench; the daemon's
startup line carries the login link; a **Remote access** settings pane turns
`tailscale serve` on, shows the `ts.net` address and a QR code that opens a
logged-in workbench on a phone; a banner on the first grilling points at it
and stays dismissed everywhere once dismissed anywhere.

## Decisions in force

All from [ADR-0015](../../adr/0015-open-boundary-and-workbench-key.md); what
bears on this stage:

- **One long-lived secret in the Data Directory**, made at first start,
  mode `0600` beside `secrets.yaml`. `/api/ui/` and the SPA's own routes
  answer 401 without a cookie carrying it. The link is the address with
  `?key=…`; the server sets the cookie and redirects to strip the query.
  **Reset key** re-issues it and logs every device out. Not a short-lived key
  per QR code.
- **What stays open**: the Share Viewer and its assets, `/api/v1/health`, and
  the Conversation-scoped session API under `/conversations/{id}/api/v1/`,
  which is a session's own. The service worker and push registration ride on
  the cookie like any page fetch — check that a push-opened URL still lands
  logged in.
- **Where the link is handed out**: the tray's Open (the desktop crate's
  `opener.rs` already opens a browser), and the startup log line. No
  `verkstead remote` subcommand and no wizard step — both rejected.
- **Remote access is a settings section** in the existing card-and-pane
  shape, added to `openings.ts`'s `WORDS` so it gets its route. Four things:
  tailscale installed and up with the node name (`tailscale status --json`,
  `.Self.DNSName`); a switch running `tailscale serve --bg 8422` on and
  taking it off, its state read from `tailscale serve status`; the resulting
  address; the QR code of the login link with a copyable link beside it, and
  Reset key under it. The QR is drawn client-side — an inline SVG from a
  small encoder, no external asset.
- **The operator grant.** `tailscale serve` from a non-root process is
  refused unless that user is the operator. The daemon shows the exact
  `sudo tailscale set --operator=<user>` for this machine's user and re-tries
  on the next press. The desktop app runs the grant through the platform's
  graphical sudo — `pkexec` on Linux, `osascript … with administrator
  privileges` on macOS, a UAC-elevated process on Windows — which is new:
  the desktop crate has no privilege escalation today. The NixOS module sets
  `services.tailscale.extraSetFlags = [ "--operator=verkstead" ]` wherever
  `services.tailscale.enable` is on.
- **The banner.** On the Conversation page above the Timeline, at every
  grilling start until dismissed, shown while the first grilling is working
  on its first Question Set. The dismissal is a server-side flag beside the
  archived switch's, read back on every load.
- **The docs.** `adoption.md`'s *tailscale serve* recipe becomes the pane;
  the first-visit story per install (tray, log line) is written where each
  install is described.

## Proposed tasks (provisional)

1. **The key and the gate.** Make and keep the secret; a middleware on the UI
   router and the SPA fallback answering 401; the `?key=` handshake and
   redirect; Reset key.
   - A request without the cookie gets 401 on `/api/ui/settings` and 200 on
     `/api/v1/health` and the share viewer.
   - A session's `curl` from inside a sandbox gets 401 (an integration test
     under `crates/server/tests/sandbox.rs`).
2. **Handing the link out.** The startup line; the tray's Open; the desktop
   crate's first-open path.
   - A fresh desktop start opens a browser that is logged in.
3. **Remote access: the pane's reads.** A `GET /api/ui/remote` view — tailscale
   present, up, node name, serve state, address, the login link.
   - A machine with no `tailscale` reads as such with the install pointer.
4. **Remote access: the presses.** Serve on and off; the operator grant with
   the three graphical-sudo arms and the daemon's shown command; Reset key.
   - Refused serve reads the command; a granted one re-tries and reads on.
5. **The pane and the QR.** Card, pane, route, the QR encoder, the copyable
   link, the module option, and tests in the settings-routes suite.
6. **The banner.** The dismissal flag and endpoint; the banner on the
   Conversation page; the grilling-start condition.
   - Dismissed on one device, absent on another after a reload.
7. **Docs and vocabulary.** CONTEXT.md gains **Workbench Key** and **Remote
   Access**; `adoption.md` per install.

## Re-verify at start

- Stage 01 landed: no Paths section beyond binds, `openings.ts`'s `WORDS` as
  it then stands.
- The router in `crates/server/src/lib.rs` still mounts `/api/ui/` and the
  SPA fallback in one place, so one middleware covers both.
- How push notifications open a URL (`web/src/push/`) and whether a
  notification-opened tab carries the cookie.
- Whether `tailscale serve status` and `tailscale status --json` still
  answer in the shape assumed, on the tailscale version in the flake.
- What the desktop crate's dialog and opener modules offer for an elevated
  run per platform.
