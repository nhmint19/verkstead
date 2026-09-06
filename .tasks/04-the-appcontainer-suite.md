# 04. The AppContainer suite

## What to build

The sibling of `crates/server/tests/sandbox_macos.rs`, asking the container the
questions that suite asks of `sandbox-exec`, and run for real on the
`windows-2025` job the way the `macos-15` job runs the seatbelt one.

**Nothing in it reads the entries the rendering wrote.** The boundary is what is
being tested, and a test that asserts the access-control list asserts itself: it
would go on passing while Windows changed what an entry meant. What settles it is
a probe inside the container attempting each access and reporting what happened,
exactly as the Mac's does.

**The vocabulary is this platform's.** A path a session may not reach is
`refused` here rather than `absent`, as on a Mac — the machine is there and
denied. The one thing to get right that neither Unix suite has to: what comes
back from a console is a drawing of a grid, so anything with a path in it is
written to a file the test reads rather than read off the Capture. The Windows
sessions suite already works this way and is the model.

Every access kind in the Surface gets a line: written where it says read-write,
read where it says read-only, refused where it says nothing, and the human's
Documents refused because the description never named them.

**Watch the job's patience.** `crates/server/tests/sessions_windows.rs` already
runs near its limit on that runner, where libtest's alphabetically-first cohort
is several times slower than the ones behind it. A container per session is more
work per test, so re-measure rather than assume, and say in the suite's own
documentation what it was measured at.

## Acceptance criteria

- [ ] Every access kind the Surface can carry is classified by attempting, and
      the classification matches what the description said for each.
- [ ] The human's Documents and the account's own skills are both refused from
      inside, and the refusal is distinguished from a path that was never there.
- [ ] The suite runs on the `windows-2025` job and its wall time is measured and
      written down, with the job's patience raised if it needs to be.
