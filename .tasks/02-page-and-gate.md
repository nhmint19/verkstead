# 02. The page and the gate

## What to build

`/setup`, and the rule that while onboarding mode is on there is nowhere else to
be.

**The gate sits around the router** rather than inside any page: while the mode
is on, every other URL redirects to `/setup`, and while it is off `/setup` is no
page at all and falls to the catch-all like any other path nothing answers. The
route table is a seam of its own here — a path that reaches no route is a page
that silently is not there — so it gets a test that walks the real route
definitions, the sibling of the one the settings panes' paths already have.

**The step is kept on the device**, in the browser storage the app's other
per-device settings live in, and never sent to the server: which step somebody
is on is a fact about the tab in front of them. Reopening the wizard opens where
they left it.

**The page reads the onboarding endpoint** and re-reads it **every ten seconds
while the current step is unmet**, stopping when it is met — the probes are
cheap, and the human is waiting for an install to land. The app already re-reads
on window focus, which covers the phone put down and picked up; nothing here
adds a second mechanism for that.

What this task draws is the wizard's frame: the three steps, which one is open,
and which stand met. What each step *contains* is the three tasks after it.

## Acceptance criteria

- [ ] With the mode on, `/`, `/compose`, `/settings` and a Conversation's URL
      all land on `/setup`; with it off, `/setup` reads as no such page and
      every other URL is untouched.
- [ ] The open step survives a reload on that device, and nothing about it
      reaches the server.
- [ ] The read re-runs every ten seconds while the open step is unmet and stops
      once it is met; a refocus re-reads whatever the interval has let go.
- [ ] A route-table test walks the real definitions for `/setup`, the way the
      settings panes' own paths are walked.
