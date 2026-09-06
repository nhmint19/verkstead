# 06. sccache as the probe said

## What to build

**Off.** The probe settled it: an sccache client inside a container panicked
reading its own configuration before it ever reached the network, and a
connection from inside to the loopback is refused anyway — which is exactly the
condition ADR-0014 named for turning the switch off.

So a sandboxed Windows session gets the shared `CARGO_HOME` and no
`RUSTC_WRAPPER`: the cache of downloads that works because it is directories,
without the cache of compiled objects that needs a server over a socket the
container cannot open.

This is a small task and deliberately so — the probe did the deciding. What is in
it beyond unsetting a variable:

- The shared Build Cache directory is granted to the container read-write, or
  `CARGO_HOME` names a directory the session cannot write and every Rust build
  inside fails at the first download.
- The Compile Server should stop being started on Windows. It serves sessions
  through `RUSTC_WRAPPER` and nothing else, so one still coming up would be a
  process nobody can reach.
- The reason is written where somebody will look for it rather than left in a
  commit message: a Windows session that builds Rust slower than a Linux one is
  a thing somebody will ask about.

## Acceptance criteria

- [ ] A session in a Rust repo builds inside its container, with the shared
      `CARGO_HOME` writable from inside and `RUSTC_WRAPPER` absent.
- [ ] No Compile Server is started on Windows, and nothing that reads one is
      left expecting it.
- [ ] Why it is off is written down where a reader meets the build cache rather
      than only in the ADR.
