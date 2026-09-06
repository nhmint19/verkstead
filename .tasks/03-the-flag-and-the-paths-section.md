# 03. The flag, the setting and the Paths section

## What to build

Everything left that *configures* a boundary goes, on both of the two sides that
used to say it. The installation's side: the `--watched-path` flag, the
environment variable behind it, and the resolve-at-startup that refused to come
up over a directory that was not there. The human's side: the watched paths in
`config.yaml`, the list the settings page saved them through, and the half of
the settings' resolution report that drew them.

This is where the boundary's own module is deleted, because this is the first
point at which nothing reads it. Tasks 01 and 02 left it standing for the
settings report and the browse; both are gone by the time this lands. The
server's public API loses the type along with it, and the router constructors
that took it lose the parameter — the router that watched nothing and the one
that watched somewhere become the same router.

**The settings page's Paths section keeps only the Sandbox Configuration's
binds.** Its card counts binds and the ones the server cannot see; its pane
edits binds; the writer that sent the two lists sends one. The empty states that
explained that no watched path was configured and so nothing could be
registered go with them — that sentence is no longer true of anything. **The
section moves below the Repos**: it sat above them because a watched path was
what a Repo was registered from, and with binds alone in it that reason is gone
and nothing replaces it.

The startup line is what a human reads to find out what the server came up as,
so it loses both of the fields that said what was watched, and the tests that
asserted on them follow. The web fixtures for the settings are written by
`cargo test` and are regenerated rather than edited.

The prose sweep — CONTEXT.md, the documents under `docs/`, and the `config.yaml`
example in the settings module's own doc comment — is task 05's, not this one's.

## Acceptance criteria

- [ ] `verkstead serve --data-dir .` starts with no other flag; neither
      `--watched-path` nor its environment variable is accepted any more, and
      the startup line names nothing watched.
- [ ] The settings save carries no watched paths, the Paths section holds
      sandbox binds and nothing else, and it is drawn below the Repos.
- [ ] Nothing under `crates/` names `WatchedPaths`, `Admission` or
      `watched_paths`, and the boundary's own module is deleted.
- [ ] `cargo test` regenerates the settings fixtures and the whole suite passes,
      web suite included.
