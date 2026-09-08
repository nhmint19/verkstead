# 01. Compose a session's PATH from the server's own

## What to build

Make the `PATH` a session gets on Linux and macOS start from the `PATH` the
server itself was started with, rather than from a list written into the
sandbox module. Read the server's `PATH` **once at startup**, compose the
session list from it, and hold that one value where both the sandbox builder
and the onboarding wizard's probes read it — the wizard's rule is *present
means a session would find it*, and two reads could disagree.

The composed list, in order:

1. Verkstead's own `bin`, as today: the executable a session asks with.
2. The server's `PATH` entries, in the order written, with three rules: the
   first occurrence of a directory wins; empty entries and relative ones such
   as `.` are dropped; and an entry that is neither under the server's home
   nor under the platform's floor — the directories the sandbox already makes
   reachable, `/nix`, `/usr`, `/bin`, `/lib`, `/etc`, `/run/current-system`
   on Linux and the Apple system list on a Mac — is dropped, because a session
   could not reach it. That last rule is what removes the `/mnt/c/...` entries
   WSL appends, and `/snap/bin` or `/opt/something/bin` with them.
3. Today's fixed list as a floor, deduplicated against what is already there,
   so a server started under a unit with a minimal `PATH` still reaches the
   machine's own toolchain.

Nothing is added that the `PATH` did not name: a server whose `PATH` lacks
`~/.local/bin` produces a session without it. Windows keeps its own arm, which
already hands a session the server's `PATH`; it neither gains nor loses an
entry from this task.

The per-user entries this lets onto the list are not yet reachable inside a
session — that is task 02. This task is the list and the probe agreeing on it.

## Acceptance criteria

- [ ] With a stub `claude` in a directory under the server's home named by the
      server's `PATH`, the wizard's Claude row reads Present and the `PATH` a
      session is given names that directory ahead of the system ones.
- [ ] A server `PATH` of `~/.local/bin:/usr/bin:/mnt/c/Windows:/usr/bin::.`
      yields a session `PATH` with `~/.local/bin`, then `/usr/bin` once, then
      the floor, and nothing else.
- [ ] The onboarding probes and the sandbox read the same composed value, and
      a test that changes the server's environment after startup sees neither
      of them move.
- [ ] A Windows session's `PATH` is what it was before this task.
