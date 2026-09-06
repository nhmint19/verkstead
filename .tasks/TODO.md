# The AppContainer

A Windows session runs inside an AppContainer: on the human's Windows 11 machine
it reaches its Worktree and asks through the pipe, and is refused their
Documents. The unsandboxed note goes, and the `windows-2025` job runs an
AppContainer suite for real, the way the `macos-15` job runs the seatbelt one.

The stage opened with a probe, which has been written and run — twice, on a real
Windows 11 machine — and what it found is written into
[ADR-0014](docs/adr/0014-windows-sessions.md) under **What the probe answered**.
Every task below reads that section rather than the documentation the ADR was
first written from: the network claims held, a pseudoconsole handed into a
container works, grants work and no ancestor needs granting, and two things came
back short of an answer and are named where they land.

Roadmap stage: [03: The AppContainer](docs/roadmaps/windows-sessions/03-appcontainer.md)

## Tasks

- [x] 01: A container runs a process — [details](01-a-container-runs-a-process.md)
- [x] 02: The rendering — [details](02-the-rendering.md)
- [x] 03: The pipe for the container — [details](03-the-pipe-for-the-container.md)
- [x] 04: The AppContainer suite — [details](04-the-appcontainer-suite.md)
- [ ] 05: Per-Conversation profiles and the sweep — [details](05-profiles-and-the-sweep.md)
- [ ] 06: sccache as the probe said — [details](06-sccache-as-the-probe-said.md)
- [ ] 07: The note goes, and the docs say what is true — [details](07-the-note-goes.md)
