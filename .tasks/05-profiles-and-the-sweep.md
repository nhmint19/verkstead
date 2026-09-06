# 05. Per-Conversation profiles and the sweep

## What to build

The lifetime of a profile, which matters because its grants are entries on the
human's real directories: granted at a Conversation's first session, removed with
its Worktree, and swept at startup for whatever a crash left behind.

One profile per Conversation, named from the Data Directory and the Conversation
id — the naming lands in task 01, and what this adds is everything that happens
to it afterwards. A session reaches its own Worktree and its own binds and no
other Conversation's.

**Closing a Conversation takes the profile with the Worktree.** The close already
removes the Worktree, the companions' worktrees and the handoff directory; the
profile and its entries go in the same place, and the entries go *before* the
profile is deleted — a deleted profile leaves its entries standing as a number
nothing can resolve.

**The server sweeps at startup**, beside the sweep that reclaims orphaned
worktrees: profiles whose Conversation is Done or Closed are deleted and their
entries stripped from the directories the Surface would have named — the repo's
git directory, the account, the `PATH` entries, Verkstead's own bin, the skills,
the binds. A Done Conversation still has its Worktree and can be sent back to
wrapping up; its next session grants what it needs again, which is why stripping
is safe.

**What can go wrong here is worse than what it fixes**, so it takes the shape the
worktree sweep takes: a reading that failed strips nothing, and every entry
removed is one this server can say it wrote.

## Acceptance criteria

- [ ] A second Conversation's Worktree is refused to the first's session,
      asserted by attempting from inside.
- [ ] Closing a Conversation leaves no entry for its profile on the account, the
      `PATH` entries, Verkstead's own bin or the skills, and the profile is gone.
- [ ] After a simulated crash — a profile and its entries left behind with the
      Conversation Done — the next startup leaves none of them.
