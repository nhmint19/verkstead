# 03. The pipe for the container

## What to build

Stage 02's named pipe, granted to the container, so that a sandboxed Windows
session can ask at all.

The pipe's listener already takes a further identity beside the account the
server runs as — the seam was landed in stage 02 with nothing passing one. This
fills it with the SID of the profile a session runs under, and the probe already
proved the shape works: a process inside a container opened a pipe whose
descriptor granted it.

**One profile per Conversation means the descriptor is not written once.** The
server opens its pipe at startup, before any Conversation has a profile, so
decide how an identity granted later reaches a pipe already open — a descriptor
rebuilt as each instance is created is the shape the listener already has, since
it makes an instance per connection.

**Loopback is refused and this is where that gets asserted.** The probe found a
connection from inside to `127.0.0.1` and to the machine's own LAN address both
time out rather than fail fast, so a test that dials either needs a deadline of
its own and must assert the refusal rather than assume it.

## Acceptance criteria

- [ ] `crates/cli/tests/sandbox_windows.rs` runs its `verkstead ask` from inside
      a container and the Set arrives, over the pipe and with nothing listening
      on TCP.
- [ ] A connection to the loopback from inside a container is asserted refused,
      with a deadline, so the assertion is about the boundary rather than about
      a test that hung.
- [ ] A container whose SID the pipe was not told about cannot open it.
