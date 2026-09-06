# 02. The rendering

## What to build

A third `Surface` renderer beside `bwrap` and `seatbelt`, chosen by Platform, so
that what a Windows session may reach is the same description the other two
render and an access-control entry is how this machine makes it true.

Each part of the vocabulary, as the probe found this platform answers it:

- **`Own` and `Elsewhere`** become a grant to the profile's SID on the real
  directory, at the Surface's reach. The probe settled that a grant inherits
  down a tree and that **no ancestor needs granting** — a container reached a
  directory deep in the human's profile with no entry above it — so the entries
  are on the paths the description names and nowhere else.
- **`Nothing`** is the account's own skills refused. **This is the one part the
  probe could not settle**: an explicit deny entry written under a granted tree
  did not refuse the path beneath it. Settle it by attempting, and the candidates
  worth trying are the deny with different inheritance or ordering, a protected
  access-control list on that one directory, or granting the account's children
  rather than the account itself. Whichever it is, the acceptance is that the
  path is refused from inside.
- **`Temporary`** is a directory of the session's own inside the fresh profile,
  granted read-write and removed when the session ends.
- **`ProcessTable` and `Devices`** are nothing at all on this platform.
- **Each `PATH` entry under the human's profile** is granted read-only, because
  a per-user tool install is not readable by a container the way Program Files
  is — the probe confirmed Program Files, the system directory and Windows
  PowerShell all run with no entry at all, so these are the only ones that need
  one. Nothing else of the profile is granted.

A junction is followed and the grant is checked on its target, which the probe
confirmed — so stage 01's fresh profile with the account junctioned into it is
granted at the real account directory.

**A container that cannot be made refuses the session** — a profile that will
not create, a grant that fails — the way a missing `bwrap` does on Linux, with a
line saying which. It never falls back to the unsandboxed session. The volume
refusal for an account that cannot be hard-linked is already there and is not
this task's.

## Acceptance criteria

- [ ] A real grilling session on the `windows-2025` job runs inside its
      container and reaches its Worktree, the Repo's git directory, its account
      and the skills.
- [ ] A path the Surface does not name — the human's Documents — is refused from
      inside, asserted by attempting rather than by reading the entries back.
- [ ] The account's own skills are refused from inside, by whichever mechanism
      this platform turns out to answer to.
- [ ] A profile that will not create and a grant that fails each refuse the
      session, saying which, with no unsandboxed fall back.
