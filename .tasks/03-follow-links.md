# 03. Follow a program's link into its install

## What to build

Claude's native installer leaves `~/.local/bin/claude` as a symlink into
`~/.local/share/claude/versions/`, and a bind of `~/.local/bin` alone gives a
session a dangling link. For **every name the wizard has a row for** — the four
harnesses, `git` and `gh`, kept in one constant so the list can grow —
resolve the name on the composed `PATH`, follow its symlink chain, and where
the file it lands on is under the server's home, bind the directory holding it
read-only as well, the way task 02 binds a `PATH` entry.

A link whose target is outside the home, or one that leads nowhere, counts as
**not found** for that name: the row reads absent, and the session is not
given a name that would fail to exec. Nothing outside the home is bound on a
link's account, which keeps the hole where task 02 left it.

And keep sessions off the updater: Claude's native binary updates itself by
writing into that versions directory, which is read-only inside, so every
Claude session's environment carries `DISABLE_AUTOUPDATER=1`. Every Claude
session, wherever `claude` was found — a session never writes into the
human's install. Codex, Grok and OpenCode sessions do not get it.

## Acceptance criteria

- [ ] A `~/.local/bin/claude` linking into `~/.local/share/claude/versions/X`
      runs inside a Linux session, and the versions directory is bound
      read-only and nothing above it.
- [ ] A `claude` link into `/opt/claude` and a dangling link both leave the
      Claude row absent, and neither directory is bound.
- [ ] The names followed are read from one constant, and adding a name to it
      is the whole of extending the rule.
- [ ] `DISABLE_AUTOUPDATER=1` is in a Claude session's environment and absent
      from a Codex session's.
