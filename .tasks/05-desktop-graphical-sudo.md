# 05. The desktop's graphical sudo

## What to build

The operator grant taken by the desktop app through the platform's own way of
asking for a password: `pkexec` on Linux, `osascript` running the command *with
administrator privileges* on macOS, and an elevated process through UAC on
Windows. This is new machinery — the desktop crate escalates nothing today, and
the whole of what it hands to the platform so far is a URL and a file to open.

**The seam is what this task is really about.** The press is in a browser, the
escalation is the app's, and the desktop crate depends on the server crate
rather than the other way round — so the app has to hand the server a way to
escalate as it starts, and a server the app did not start simply has none. That
is the shape: one behaviour with two arms, picked by what the process was
started as rather than by which platform it is on. The arm with no way to
escalate is the daemon's from the task before — show the command, re-try on the
next press.

A dialog somebody cancels is not a failure to report as one. The switch stays
off, the daemon's command stays on the pane, and there is still a way through
for somebody who would rather type it.

## Acceptance criteria

- [ ] Pressing the switch without the grant, in the desktop app, raises the
      platform's own password dialog.
- [ ] Cancelling it leaves the switch off with the command still shown; granting
      it re-tries and serves.
- [ ] A server the desktop app did not start escalates nothing and shows the
      command, exactly as it did before this task.
- [ ] Each platform's arm is tested on the command it would run.
