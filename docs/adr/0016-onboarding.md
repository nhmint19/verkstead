# Onboarding

Builds on [ADR-0015](0015-open-boundary-and-workbench-key.md): the wizard is
designed for a Verkstead with no Watched Path and a workbench key.

A fresh Verkstead is unusable until four things are true, and today nothing
says which of them is false. The server never checks for `bwrap`, `git` or an
agent binary — a missing one is a session that silently fails to start,
logged and nothing more. Nothing reads `~/.claude` to offer a Profile, nothing
reads `~/.gitconfig` to offer an author, and the compose page draws an empty
Repo dropdown with no word about why. The adoption docs carry the whole of the
setup as prose. This decision replaces that prose with an **onboarding mode**
the server enters at startup, and gives the compose page a zero state and a
way to make a repository.

## Onboarding mode

**Evaluated once at startup.** The server computes whether the objective is
met — a sandbox, `git` and at least one harness present; at least one Agent
Profile; a git author — and enters onboarding mode where it is not. The mode
stays on until the wizard finishes and never re-enters until the next start:
deleting the last Profile mid-run is the settings page's empty state to say,
not a return to the wizard. There is no skip. The GitHub token is the one
thing the wizard asks for that does not gate it, because GitHub may not be in
use at all; git is not optional, so the author is.

**One page, one endpoint.** The wizard is `/setup`, the current step kept on
the device; every other URL redirects there while the mode is on. `GET
/api/ui/onboarding` is its model: the mode, the platform and distro, each
dependency's presence, and which steps stand met. It probes on every read —
the probes are cheap — and the wizard re-reads it every ten seconds while a
step is unmet and on window focus, which the app already does. Nothing runs
when nobody is looking. Nothing is folded into the settings view: the verdict
is a different question from what is configured.

**Present means a session would find it.** A session resolves `claude`, `git`
and the rest on the PATH inside the Sandbox, not the server's, and on Linux
that PATH did not include `/usr/local/bin`, where `npm install -g` on the
Debian family puts a binary. It does now, and every probe resolves on that
PATH. Claude's native installer lands in `~/.local/bin`, which is still off
it, so its instruction says to use a system prefix or a symlink. The Linux
sandbox row is a trivial `bwrap` run rather than a PATH lookup, because
unprivileged user namespaces can be off and the AppImage cannot carry
`bwrap`; a failure's stderr is shown under the row. On macOS `sandbox-exec`
is on every Mac and the row is simply ticked; on Windows it reads *not
applicable* with the existing unsandboxed note and passes. `gh` is an
optional row: shown present or absent with its instruction, never gating.

**Instructions per OS.** The distro is read from `/etc/os-release`, `ID` then
`ID_LIKE`; the detected tab opens and the others stay reachable in case it is
wrong. macOS, Windows, NixOS, Ubuntu, Fedora, Debian and Arch — each with its
derivatives — get exact commands for `bwrap` and `git`; a harness gets the
exact command where the OS has a package for it (nixpkgs, Homebrew, npm) and
a link to the vendor's install page where not, each saying where the binary
must land. Any other Linux gets the generic list of what is needed. The step
waits for **Continue** — it never advances under somebody's hands.

**Accounts are detected in the server's HOME**: `~/.claude` with
`~/.claude.json`, `~/.codex`, `~/.grok`, and opencode's XDG pair; on Windows
under `%USERPROFILE%`. Each is offered ticked where its harness is present,
listed unticked and greyed where not. A ticked account becomes a Profile with
a **null name** and every model this build knows for that harness, both
editable later on the settings page. A name is for telling two accounts of
one harness apart, which is rare, so at most one Profile per harness may be
unnamed, and an unnamed one shows no name wherever the harness mark and the
model already say enough — reading *Default* only where a name must be shown.
Nothing found says what to run to make one (`claude` once, then log in),
keeps probing on the same cadence, and offers the manual form under it.

**The git step asks for what git needs**: author name and email, required,
and the token, optional. The settings module's rule is *told, not found*, and
a prefill the human confirms is still telling: name and email come from the
server's own `git config --global`, the token from `GH_TOKEN` or
`GITHUB_TOKEN` in the server's environment and then from the host `gh`'s own
login, each labelled with where it was found and none saved until Continue.
The GitHub login is shown from verifying the token, never typed. Saving goes
through the settings save the page already has, verification included.

## The zero state

The wizard ends on the compose page, and the compose page has a **zero
state** that is a fact about the list rather than about onboarding: whenever
the sidebar's list *as filtered* is empty — no unarchived Conversation, and
archived ones either absent or hidden — the sidebar is not drawn, `/`
redirects to `/compose`, the wordmark and the gear take the pane title's
place, and the archived switch is pinned to the page's bottom-left, hidden
when nothing is archived. Switching it on with archived Conversations present
brings the sidebar back with them in it, because a switch pinned to a page
has to show something when pressed. The settings page keeps its sidebar.

## Repositories that do not exist yet

A fresh install has no Repo to compose against, so the Repo dropdown grows two
rows at its foot, behind a rule: **Create repo** and **Open repo**, each a
modal, serving the compose page and a draft's composer alike — one control,
so a draft moved onto a repo made there is the same move as picking a
registered one.

Create takes a parent directory and a name. The parent is browsed from the
server's `HOME`, which is where ADR-0015 starts an unbounded browse and the
only answer a first run has, there being no last parent to remember yet. It
makes the directory, runs `git init` on `main`, and commits a `README.md`
holding the name as the configured author — an empty repository has a default
branch but no commit, and a Conversation cannot take a base from one. When a
token is saved it offers **Create on GitHub too**, ticked and private,
through `gh repo create` after the initial commit; without a token it says a
remote is needed before the work is finished, since the pipeline ends in a
push and a pull request. Open is the registration form's path field in a
modal, browsing anywhere with repositories marked, refusing as the settings
page does. Either way the Repo is registered and becomes the draft's.

## Considered Options

- **A live onboarding predicate**, re-entering the wizard whenever the
  objective stops being met. Rejected: deleting a Profile mid-run would drop
  the human into a first-run page over work they have.
- **A skip.** Rejected: nothing runs short of the objective, and the empty
  states it would fall through to are the ones the wizard exists to replace.
- **Gating on the token and the author too**, or on neither. Rejected for
  the shape above: git is a dependency and its author is what git asks for;
  GitHub is a choice.
- **Probing on the server's PATH and binding what was found into every
  sandbox.** Rejected: a `node`-based install needs `node` bound in too, and a
  directory of the human's read-only in every session is a hole.
- **Probing on the sandbox PATH as it was.** Rejected: only nix and the
  Fedora and Arch distro packages land on it.
- **A server timer that nudges the page.** Rejected: the probes are cheap
  and the page already re-reads on focus; probing on read means nothing runs
  unwatched.
- **Advancing a step by itself when its rows tick.** Rejected as jarring.
- **A separate GitHub username field.** Rejected: the login is a fact the
  token verification already returns.
- **An empty initial commit**, or none. Rejected: a README is what a fresh
  repository conventionally holds, and no commit leaves the draft without a
  base.
- **A projects-directory setting** in place of the paths step. Rejected: the
  modal asks for a parent directory, and nothing else needed one.
