# 03. The onboarding wizard

## Goal

A fresh Verkstead sets itself up. Demonstrable: a bare start on a machine
with no `bwrap` opens on `/setup`, names the missing dependency with this
distro's install command, ticks it within ten seconds of the install landing,
and Continue leads through accounts and git to the compose page; the same
start on a machine that has everything and one Profile and an author opens
the workbench as it always did.

## Decisions in force

All from [ADR-0016](../../adr/0016-onboarding.md); what bears on this stage:

- **The objective**: a sandbox, `git` and at least one of the four harnesses
  present; at least one Agent Profile; a git author. The token does not gate.
  **Evaluated once at startup**; the mode stays on until the wizard finishes;
  no skip; no re-entry until the next start.
- **`GET /api/ui/onboarding`** is the model: mode, platform, distro, each
  dependency row's state, each step's met-ness. Probes on every read; the
  wizard re-reads every ten seconds while a step is unmet, and the app's
  focus re-read covers the refocus. Not folded into the settings view.
- **`/setup`** is the page, the step kept on the device; every other URL
  redirects there while the mode is on — gated in `App()` around the router,
  and a route-table case in `settings-routes.test.tsx`'s sibling for it.
- **Present means a session would find it.** Probes resolve on the sandbox's
  PATH (`LINUX_PATH` gains `/usr/local/bin`; `APPLE_PATH` as it is; Windows
  through `open::found`), lifting `on_the_path` out of `build_cache`. The
  Linux sandbox row runs `bwrap --ro-bind / / /bin/true` and shows stderr on
  failure; macOS ticks; Windows reads *not applicable* with the existing
  `unsandboxed` note's wording. `gh` is an optional row. All of it follows
  `platform.rs`'s discipline: `Platform` as a value and the environment read
  once, so every arm is unit-tested on one runner.
- **The distro** is `/etc/os-release`, `ID` then `ID_LIKE`, into one of
  macOS, Windows, NixOS, Ubuntu, Fedora, Debian, Arch, or *other Linux*;
  the detected tab opens and the others stay reachable.
- **Instructions**: exact commands for `bubblewrap` and `git` per OS; a
  harness gets the exact command where the OS has a package (nixpkgs has
  `claude-code`, `codex` and `opencode`; Homebrew and npm have most) and a
  vendor link where not; each says where the binary must land, and Claude's
  native installer's `~/.local/bin` is called out as off the sandbox PATH.
  On NixOS with the module, `bwrap` is on the unit's PATH already and a
  harness goes in `environment.systemPackages`. Continue is a press.
- **Accounts** are detected in the server's HOME (`%USERPROFILE%` on
  Windows) in the four shapes `sandbox.rs`'s `account_inside` knows; ticked
  where the harness is present, greyed where not. A ticked one saves through
  the existing profile create with a **null name** and every model
  `KNOWN_MODELS` lists for the harness. Nothing found says what to run,
  keeps probing, and offers the manual form under it. **Continue needs at
  least one Profile** — ticked or made in the form — because otherwise the
  step walks past one of the three things the objective is, and the mode
  clears onto the empty state a skip was rejected for.
- **The unnamed Profile** is this stage's store change: the name column
  becomes nullable, uniqueness becomes *at most one unnamed per harness*
  and *no two named alike*, and every reading of a Profile's name — the
  pairing rows, the settings card, the session record — shows nothing where
  the mark and model suffice and *Default* where a name must be shown.
- **The git step**: name and email required, token optional; prefilled from
  `git config --global` run by the server, `GH_TOKEN` then `GITHUB_TOKEN`
  in the server's environment, then the host `gh`'s login (`gh auth token`),
  each labelled with its source; nothing saved until Continue; saved through
  the existing whole-settings save, whose verification returns the login
  the step then shows.
- **After the last step** the mode is off for this run and the app lands on
  `/compose`. Stage 04 makes that page's zero state; until it lands the
  compose page is the one that exists.

## Proposed tasks (provisional)

1. **The verdict.** `onboarding.rs`: the objective, the platform and distro
   reads, the dependency probes on the sandbox PATH, the account detection;
   the mode as server state set at startup and cleared at finish; the
   endpoint and its wire type.
   - Every arm exercised on one runner through `Platform` values and a fake
     environment and PATH.
2. **The sandbox PATH.** `/usr/local/bin` on Linux; the probe helper lifted
   from `build_cache`.
   - The sandbox integration test sees `/usr/local/bin` on a session's PATH.
3. **The unnamed Profile.** Store migration to a nullable name; the two
   uniqueness rules; the render types; every reading in the workbench.
   - Two unnamed Claude Profiles are refused; an unnamed Claude and an
     unnamed Codex are not.
4. **The page and the gate.** `/setup`, the redirect while the mode is on,
   the step kept on the device, the polling read.
5. **The dependencies step.** Rows, the distro tabs, the instruction
   content, the ready state, Continue.
6. **The accounts step.** Detected rows with ticks, the nothing-found state
   and its probing, the manual form under it, the autoconfigured save, and
   Continue held until a Profile exists.
   - Unticking every detected account leaves Continue refused, and saving one
     from the manual form releases it.
7. **The git step and the finish.** Prefill with sources, the save, the
   verification's login, the last Continue clearing the mode.
8. **Docs and vocabulary.** CONTEXT.md gains **Onboarding Mode** and amends
   **Agent Profile** for the unnamed case; `adoption.md`'s *once per machine*
   becomes the wizard; the desktop roadmap's *first-run is the browser's job*
   line now points here.

## Re-verify at start

- Stages 01 and 02 landed: no boundary in the profile checks, `/setup` sits
  behind the key.
- `sessions.rs`'s `binary()` and `sandbox.rs`'s `LINUX_PATH` / `APPLE_PATH`
  are still where a session's binary is resolved from.
- `web/src/models.ts`'s `KNOWN_MODELS` still carries the per-harness list
  the autoconfigured Profile takes.
- The profile create endpoint's refusals (`Nameless`, `Modelless`,
  `NameTaken`) — `Nameless` goes with the nullable name.
- The install commands themselves: package names on each distro, and whether
  Homebrew and npm still carry each harness under the names assumed.
