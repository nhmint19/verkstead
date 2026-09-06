# 01. A container runs a process

## What to build

The mechanism, before any description is translated into it: an AppContainer
profile with an identity of its own, and a process really started inside one —
on the pseudoconsole a session runs on, and off a console for everything that
reads what a process printed rather than watching it.

The profile is named from the Data Directory and the Conversation id, so it is
the per-Conversation identity from the first moment rather than a throwaway that
a later task has to replace. Its capability is the internet client and nothing
else. The security capabilities go on the **same attribute list** stage 01's
`CreateProcessW` already carries the pseudoconsole on — a list is one block of
memory sized for the number of attributes it holds, so this widens the existing
one rather than adding a second.

**Sessions are not switched over in this task.** A container with no grants
reaches nothing, so a session moved into one here would be a session that cannot
read its own Worktree. What this delivers is the mechanism, proved by attempting
it; the description arrives in 02 and is what makes a real session work.

**The seam is the thing to get right.** What crosses from the sandbox to the
spawn is a `Rendering`, and it has nowhere to carry a container identity today —
and its `From<&Rendering> for Command` conversion, which the Compile Server and
the sandbox test suites both use, cannot start a process in a container at all.
So this task decides how the identity travels and provides the off-console
start: tasks 03, 04 and 06 each need to run something inside a container and
read what it said, and none of them can do it through the standard library.

`crates/server/examples/appcontainer-probe.rs` is a working prototype of every
Win32 call this needs — profile creation, the capability attribute beside the
pseudoconsole, and the handle plumbing — written against the same `windows-sys`.
Read it before writing this, and delete nothing of it: it is what a later
question about this machine gets asked with.

## Acceptance criteria

- [ ] A process started on a Verkstead pseudoconsole from inside a container
      reaches the Screen, and the assertion that it is really inside one is made
      off its token carrying the profile's SID rather than assumed.
- [ ] The Job Object still kills it, and the profile is deleted when the thing
      that owned it has gone.
- [ ] The same rendering started off a console has what it printed read back,
      which is what the suite, the ask test and the Compile Server stand on.
- [ ] A profile that will not create is an error that says so, not a panic and
      not a silent fall back to a process outside a container.
