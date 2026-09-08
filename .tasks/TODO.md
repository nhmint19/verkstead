# Harness PATH order

A session finds `claude`, `codex`, `grok`, `opencode`, `git` and `gh` on the
`PATH` the server itself was started with, in the order it was written, ahead
of the machine's fixed list — so the harness the human actually prefers is the
one a session launches. Today a session's `PATH` on Linux and macOS is a fixed
list of system directories, and a Claude Code installed the vendor's way into
`~/.local/bin` is a program that runs in a terminal and is not there for a
session at all. On Ubuntu under WSL the distribution's own package was too old
to connect, and the native install was the only current one.

Settled by the grilling: the server's own `PATH` env, read once at startup and
shared by the sandbox and the wizard's probes; Verkstead's `bin` first, then
the server's entries, then today's fixed list as a floor; first occurrence
wins, empty and relative entries dropped, and any entry neither under the
server's home nor under the platform floor dropped since a session could not
reach it. Nothing is added the `PATH` did not name. Every remaining per-user
entry is bound read-only, the rule Windows already applies. For every name the
wizard has a row for — one constant, to grow — the symlink chain is followed
and the directory it lands in is bound read-only where under the home; a
target elsewhere or a dangling link is not found. Every Claude session gets
`DISABLE_AUTOUPDATER=1`. The wizard row shows the resolved path and the link
target, an unreachable find reads absent with a note, the tab draws the real
list from the wire, and the Claude row leads with the native installer on
Linux and Windows. The NixOS module and the Mac app started from the Dock are
follow-ups. ADR-0016 is amended in place.

## Tasks

- [ ] 01: Compose a session's PATH from the server's own — [details](01-servers-path.md)
- [ ] 02: Reach per-user directories inside a session — [details](02-per-user-binds.md)
- [ ] 03: Follow a program's link into its install — [details](03-follow-links.md)
- [ ] 04: The wizard says where it found each program — [details](04-wizard-says-where.md)
- [ ] 05: Instructions that match — [details](05-instructions.md)
