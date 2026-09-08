# 04. The wizard says where it found each program

## What to build

With harnesses resolved in `PATH` order, which one won matters: a stale system
`claude` shadowing a current one in `~/.local/bin` is the exact bug this work
is for. Make the wizard's dependency step say where.

A row that is present carries the path the name resolved to on the session's
`PATH`, and, where that path is a symlink, the file it finally lands on. The
viewer draws the resolved path under the row and the target after it where
the two differ, so the Claude row reads `~/.local/bin/claude` and its versions
target.

A name that is on the server's *raw* `PATH` somewhere the composition in task
01 dropped — `/opt/foo/bin`, a `/mnt/c/...` entry — reads **absent with a
note** saying where it was seen and that a session cannot reach it, using the
row's existing trouble text. A link into somewhere outside the home, or a
dangling one, says so the same way. Absent with no note stays what it is: not
on the `PATH` at all.

And put the real list on the wire: the tab's *where a session looks* line
today is prose naming the fixed list, and the server now knows the composed
`PATH`. Carry it in the onboarding model and draw it, on every platform,
replacing the prose landing line.

## Acceptance criteria

- [ ] A present row shows the resolved path, and the link target where it
      differs; a row resolved to a plain file shows the path alone.
- [ ] A `claude` under `/opt/foo/bin` on the server's `PATH` reads absent with
      a note naming that path; a `claude` on no `PATH` entry at all reads
      absent with no note.
- [ ] The dependency tab on Linux and macOS lists the composed session `PATH`
      as the server holds it, and the Windows tab lists the server's.
- [ ] The onboarding view's TypeScript types are regenerated and the viewer's
      suite covers the drawn path, target and note.
