# 02. Reach per-user directories inside a session

## What to build

A directory on a session's `PATH` is worth nothing unless the sandbox can
read it, and on Linux `~` inside a session is an empty home with only the
account mounted in, while a Mac's policy denies by default. Bring the rule the
Windows boundary already applies to the other two platforms: **every entry of
the session's `PATH` that sits under the server's home is granted read-only.**

On Linux that is a read-only bind of the host directory onto the same path
inside; on macOS it is an allow rule for reading, mapping and executing under
that path. Render it from the sandbox's description the way every other grant
is, so the three platforms read one description and the Windows arm keeps
doing what it does. The grants go in ahead of the account and the refusal over
the account's own skills, so what is said later still stands over them.

The hole stays bounded to per-user directories the `PATH` names: nothing
outside the home becomes readable, and an entry the composition in task 01
dropped is not bound. A directory named on the `PATH` that does not exist is
skipped rather than refused — a stale entry in somebody's shell profile is not
a reason a session cannot start.

## Acceptance criteria

- [ ] A stub `claude` script in a directory under the server's home, named by
      the server's `PATH`, runs inside a Linux session launched the ordinary
      way, and a macOS policy rendered for the same description carries a
      read rule for that directory and no write rule.
- [ ] Nothing outside the server's home is newly readable, and the refusal
      over the account's own skills still holds with a per-user grant present.
- [ ] A `PATH` entry under the home that does not exist neither refuses the
      session nor appears among the binds.
- [ ] The Windows grant entries for the same description are unchanged.
