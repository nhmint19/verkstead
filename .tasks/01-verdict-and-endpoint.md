# 01. The verdict and the endpoint

## What to build

A server that knows whether it can do anything yet, and one endpoint that says
so.

**The objective** is three things: a sandbox, `git` and at least one of the four
harnesses present; at least one Agent Profile; a git author. The GitHub token
does not gate — GitHub may not be in use at all, while git is not optional and
its author is what git asks for.

**Evaluated once, at startup.** Where the objective is unmet the server enters
**onboarding mode**, which is state of its own beside the rest of what a run
holds. It stays on until the wizard finishes and never re-enters until the next
start. There is no skip.

**`GET /api/ui/onboarding` is the model**: the mode, the platform, the distro,
each dependency row's state, and whether each of the three steps stands met. It
**probes on every read** — the probes are cheap, and probing on read means
nothing runs while nobody is looking. Nothing here is folded into the settings
view: the verdict is a different question from what is configured.

**Present means a session would find it.** A session resolves its binaries on
the PATH inside the Sandbox rather than the server's, so every probe resolves on
that same PATH. Two things follow:

- The Linux sandbox PATH gains `/usr/local/bin`, where `npm install -g` on the
  Debian family puts a binary. The Apple one already has it; Windows has no such
  list and resolves through the open rendering's own lookup, which is what reads
  `PATHEXT`.
- The PATH lookup itself is lifted out of the build cache, which is where it
  lives today, into somewhere both callers can reach.

**The sandbox row is a run rather than a lookup** on Linux: `bwrap --ro-bind /
/ /bin/true`, because unprivileged user namespaces can be off and the AppImage
cannot carry a `bwrap`. A failure keeps its stderr for the row to show. On macOS
`sandbox-exec` is on every Mac and the row simply ticks; on Windows it reads
*not applicable* and passes. `gh` is a row of its own and never gates.

**The distro** is `/etc/os-release`, `ID` and then `ID_LIKE`, resolved into one
of macOS, Windows, NixOS, Ubuntu, Fedora, Debian, Arch, or *other Linux*.

All of it follows the platform module's discipline: the platform is a value
rather than a `cfg`, and the environment is read once and passed down, so every
arm — including the two this runner will never be — is a unit test on this one.

Accounts are not this task's: the wizard's accounts step brings its own reads.

## Acceptance criteria

- [ ] A start on a machine with no `bwrap` comes up in onboarding mode, and the
      read names the sandbox row as the unmet one with the failed run's stderr
      under it.
- [ ] A start with a sandbox, `git`, a harness, one Profile and an author comes
      up with the mode off, and the read says so.
- [ ] Every platform and distro arm is exercised on this one runner through
      platform values and a fake environment and PATH — the Windows lookup's
      `PATHEXT` handling and every `ID` / `ID_LIKE` mapping included.
- [ ] A session's PATH holds `/usr/local/bin` on Linux, proved the way the
      sandbox suite proves everything else: by running a probe inside a session
      rather than by reading the constant it was built from.
- [ ] The verdict is fixed at startup — deleting the last Profile while the
      server runs leaves the mode off, and the endpoint still reports the step
      as unmet.
