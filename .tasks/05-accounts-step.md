# 05. The accounts step

## What to build

The wizard's second step: the agent accounts already on this machine, offered as
Agent Profiles.

**Detected in the server's own HOME** — `%USERPROFILE%` on Windows — in the four
shapes an account comes in: Claude's directory and the config file beside it,
Codex's home, Grok Build's home, and opencode's XDG pair. One place in the
sandbox module already says what each shape is made of, and this reads that same
list rather than writing a second one; a backend arriving later then shows up
here without being taught to twice. Because each shape is a fixed path under
HOME, at most one account per harness is ever found — which is exactly why the
unnamed rule is per harness.

**Each is offered ticked where its harness is present**, and listed unticked and
greyed where it is not: an account whose binary is missing is not something to
make a Profile of yet.

**A ticked one saves through the existing profile create**, with a **null name**
and every model this build knows for that harness. Both are editable later on
the settings page; nothing here asks the human to type either.

**Nothing found** says what to run to make an account — `claude` once, then log
in — keeps probing on the same ten-second cadence as the step before it, and
offers the manual Profile form under it.

**Continue needs at least one Profile**, ticked here or made in the form. It is
one of the three things the objective is, and the only one a step could
otherwise walk past: clearing the mode with nothing saved would land the human
on exactly the empty state a skip was rejected for, until the next start put
them through the wizard again.

## Acceptance criteria

- [ ] Each of the four shapes present in the server's HOME is offered — ticked
      where its harness is present, greyed where not — and the same reading
      works from `%USERPROFILE%` on Windows.
- [ ] Saving a ticked account makes a Profile with no name and every model this
      build knows for that harness, through the profile create the settings page
      already uses.
- [ ] With nothing found the step says what to run, offers the manual form, and
      picks an account up within ten seconds of it appearing.
- [ ] Unticking everything leaves Continue refused; saving one Profile from the
      manual form releases it.
