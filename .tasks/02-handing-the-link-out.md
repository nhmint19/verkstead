# 02. Handing the link out

## What to build

The login link put in front of a human, in the two places an install has for it.

**The daemon's way is the startup line** — the one that already says where it is
listening and which Data Directory it resolved. It carries the address with
`?key=`, so somebody reading the journal has a link they can paste.

**The desktop's way is the browser it already opens.** A fresh start opens the
viewer on the login link, and the tray's **Open** goes to the same link rather
than to the bare address, so a later press lands logged in whatever the browser
has since forgotten.

Neither a `verkstead` subcommand nor a wizard step: the first would be a second
place to print one line, and the second would put a step nothing gates in front
of somebody who has not yet seen a Conversation.

**The seam to solve.** The desktop builds the viewer's URL *before* it spawns
the server, and the Data Directory is only resolved inside the server's own
startup. So either the app resolves the directory and asks for the key before
serving starts, or making the key moves somewhere both halves call. Whichever
way, the app and the server must arrive at the **same** secret — a desktop that
made one of its own would lock itself out of the Data Directory it restarts
against.

And prove the notification path: a service worker navigates same-origin and the
page's own requests carry the browser's default same-origin credentials, so a
push-opened URL should already land logged in. Nothing is expected to change
here; the test is what says so.

## Acceptance criteria

- [ ] The server's startup line carries the address with `?key=`, and pasting it
      into a browser lands in a logged-in workbench.
- [ ] A fresh desktop start opens a browser that is logged in, and the tray's
      **Open** lands logged in on a later press too.
- [ ] A desktop restarted against the same Data Directory hands out the key the
      first run made, rather than issuing a second one.
- [ ] A URL opened from a push notification lands logged in.
