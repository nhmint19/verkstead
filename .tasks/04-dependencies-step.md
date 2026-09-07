# 04. The dependencies step

## What to build

The wizard's first step: what this machine is missing, and the exact command
that fixes it here.

**The rows** are the sandbox, `git`, the four harnesses, and `gh`. The sandbox
row shows the failed `bwrap` run's stderr under it where there is one, ticks on
macOS, and reads *not applicable* on Windows in the words the unsandboxed note
already uses. The `gh` row is drawn present or absent with its instruction and
**never gates** — nothing Verkstead does short of GitHub needs it. The objective
wants a sandbox, `git` and *at least one* harness, so three harness rows left
unticked hold nothing up.

**The distro tabs** are macOS, Windows, NixOS, Ubuntu, Fedora, Debian, Arch and
other Linux. The detected one opens; the other seven stay reachable, in case the
detection is wrong.

**The instructions** are exact commands for `bubblewrap` and `git` on each OS.
A harness gets an exact command where that OS has a package for it, and a link
to the vendor's install page where it does not:

- nixpkgs has `claude-code`, `codex` and `opencode`.
- Homebrew has `opencode` as a formula, and `claude-code` and `codex` as
  **casks** — `brew install --cask`, not a plain `brew install`.
- npm has `@anthropic-ai/claude-code`, `@openai/codex` and `opencode-ai`.
- **Grok Build gets a vendor link on every OS.** The `grok-cli` in nixpkgs and
  the one on npm are other people's projects — an agent of superagent-ai's and a
  proxy around claude-code — rather than xAI's `grok`, and offering either would
  install the wrong program under the right name.

Each instruction says **where the binary has to land**, and Claude's native
installer's `~/.local/bin` is called out as off the sandbox PATH: a system
prefix or a symlink is what makes it a binary a session can find. On NixOS with
the module, `bwrap` is already on the unit's PATH and a harness goes in
`environment.systemPackages`. Any other Linux gets the generic list of what is
needed.

**Continue is a press.** The step never advances under somebody's hands, however
the rows tick, and it is refused while a gating row is unmet.

## Acceptance criteria

- [ ] The detected distro's tab opens and all eight stay reachable.
- [ ] The Linux sandbox row shows the failed run's stderr; macOS ticks; Windows
      reads *not applicable* in the existing note's wording; `gh` reads present
      or absent and never holds Continue.
- [ ] Every OS names an exact command for `bubblewrap` and `git`, an exact
      command for each harness that OS packages, and a vendor link for the rest
      — Grok Build's being a link on all of them — with Claude's `~/.local/bin`
      caveat on its row.
- [ ] With a gating row unmet Continue is refused; installing what was missing
      ticks that row within ten seconds and releases Continue, and nothing
      advances until it is pressed.
