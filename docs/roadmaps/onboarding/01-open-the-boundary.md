# 01. Open the boundary

## Goal

The Watched Path is gone from Verkstead. Demonstrable: a bare `verkstead
serve --data-dir .` with no flags registers a git repository root anywhere the
server can read and saves an Agent Profile over `~/.claude`; its path fields
open at the server's `HOME` rather than at `/`; the settings page's Paths
section holds sandbox binds and nothing else; the NixOS VM test registers a
repo the unit was told to bind and reports one it was not as *missing*, naming
the `paths` option; `nix flake check` passes with the module built without a
minimum.

## Decisions in force

All from [ADR-0015](../../adr/0015-open-boundary-and-workbench-key.md); what
bears on this stage:

- **The concept goes everywhere, not just from the settings.** No
  `--watched-path` flag, no `VERKSTEAD_WATCHED_PATHS`, no admission on repo
  registration, on a Profile's account, or on the path browser — every field
  browses anywhere the server can read, which `BrowseScope::Anywhere` already
  is. Keeping it as an installation-only flag was rejected as two behaviours
  for one boundary stage 02 replaces.
- **The unbounded browse opens at `HOME`.** `BrowseScope::Watched` with no
  path was seeded from the watched roots, and `Anywhere`'s no-path case is
  `topmost()` — `/`, or the drive list on Windows. Every field the boundary
  used to seed would open at the root once they are one scope, so the no-path
  ask answers the server's own `HOME` instead. A starting point and not a
  boundary: the listing walks up out of it like any other, and a `HOME` the
  server cannot read falls back to `topmost()`.
- **What is left of the rule is the Sandbox.** A session reaches the Worktree,
  the Repo's git directory and the account, composed from what the
  Conversation names. Nothing about that composition changes here.
- **Refusals that stay.** A Repo path must be absolute and a repository root
  with a default branch; an account must be a directory (or Claude's pair) of
  its harness's shape. `NotAbsolute`, `Missing`, `NotARepository`,
  `NoDefaultBranch` and the per-field account refusals stay; the
  `OutsideWatchedPaths` arms go, on registration and on `Broken`.
- **The NixOS module's option is renamed and re-meant.** `watchedPaths`
  becomes `paths`: directories bound read-write into the unit, repositories
  and accounts alike, no minimum, the assertion gone. It has no Verkstead
  meaning. `home` stays `BindReadOnlyPaths`; an account under it that a
  session must write is named in `paths` as well, and the docs say so. The
  missing-path refusal on that install says which option to name it in — the
  server knows it is the nix install the way `adoption.md` already describes
  the namespace report.
- **The settings' resolution report goes with its section.** `PathsView`
  keeps the binds' `PathSource` and `PathResolution`; the watched half of it
  is deleted rather than emptied.
- **The security cost is accepted and named.** The UI API is reachable from a
  session over the loopback and was before; stage 02 is what closes it. Do
  not compensate for that here.
- **Vocabulary.** CONTEXT.md's **Watched Path** entry is retired; **Repo**
  and **Agent Profile** lose the sentences that say they are registered from
  within one; **Sandbox Configuration** loses *configured where the Watched
  Paths are*. `adoption.md`, `development.md` and the design doc follow.

## Proposed tasks (provisional)

1. **Server: registration and accounts without the boundary.** Drop the
   `WatchedPaths` parameter from repo registration, profile checks, `broken()`
   and the pairing readers; delete `watched.rs` and the `Outside` arms.
   - Registering a repository root outside any directory named at startup
     succeeds; a relative path and a non-root are still refused by name.
   - A Profile over the server's own `~/.claude` saves and reads unbroken.
2. **Server: startup and browsing.** Remove the flag, the env var and the
   resolve-at-startup; collapse the browser to the one unbounded scope, seed
   its no-path ask from `HOME`, and drop the `scope` query parameter from the
   wire type.
   - `verkstead serve --data-dir .` starts with no other flag and the startup
     line no longer says `watched`.
   - A browse with no path lists the server's `HOME`, and one with a path
     above it lists that, so nothing is out of reach.
3. **Workbench: the Paths section and the path fields.** The Paths card and
   pane hold sandbox binds only; `PathField` loses `scope`; the repo and
   profile forms browse anywhere; the `heldPaths` writer sends binds alone.
   - The settings save no longer carries `watched_paths`, and the fixtures
     `cargo test` writes for the web suite are regenerated.
4. **NixOS module and VM test.** Rename to `paths`, drop the assertion,
   bind read-write, document the account case; the VM test asserts the
   *missing* refusal names the option.
   - `nix build .#checks.x86_64-linux.vm` passes.
5. **Docs and vocabulary.** CONTEXT.md, `adoption.md`, `development.md`, the
   design doc's product decisions, and the config example in `settings.rs`'s
   module doc.
   - No document under `docs/` names a Watched Path except this roadmap and
     the ADRs.

## Re-verify at start

- The 24 Rust files referencing the boundary type (`grep -rn "WatchedPaths\|watched::\|state\.watched\|Admission::" crates`) — the count, and whether any use landed since this was staged.
- `nix/module.nix`'s `BindPaths` block still derives from `cfg.watchedPaths`
  and `BindReadOnlyPaths` from `cfg.home`.
- `crates/server/tests/ui_content.rs` still writes the web fixtures, so the
  settings fixtures are regenerated by `cargo test` rather than by hand.
- Whether anything besides repos, profiles and browsing has come to consult
  the boundary — companions, attachments and the terminals were clean when
  staged.
