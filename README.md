# Verkstead

A management platform for agentic coding. Everything is driven from a web GUI:
a background orchestrator creates worktrees, runs and monitors sandboxed coding
sessions, puts question sets and commits to you, and works through task lists
and staged roadmaps unattended.

Verkstead (Norse *verk*, work + *stead*: a workshop) began as a clone of
[askance](https://github.com/tobico/askance) and keeps its architecture — a
Rust workspace and a SolidJS SPA in one binary, SQLite, SSE nudges, web push,
and server-side rendering of everything an agent writes. askance remains a
separate, maintained product; Verkstead diverges freely from it.

## Status

**Early, private, and unreleased.** There is nothing to download yet. The
workbench, the sandboxed sessions, the task-list and roadmap pipelines and the
per-PR wrap-up are built; what has not happened yet is a real repository driven
through them end to end, which is where [adoption](docs/adoption.md) stands.

What a tag will produce is four ways in rather than one, and no two of them
are the same thing. On a host that is always on, the flake builds the headless
daemon and the NixOS module runs it. On a Linux desktop,
`Verkstead-x86_64.AppImage` is that same server started from an icon: the viewer
in your browser and a tray icon over it — or no icon at all where the desktop
has no tray host, vanilla GNOME being the case people meet, and it serves just
the same. On a Mac, `Verkstead-universal.dmg` carries `Verkstead.app` — one
download for both Macs, the same server again, with its icon in the menu bar.
That app is unsigned, so the first launch is refused and System Settings is
where it is allowed through; the steps are written out beside the download in
[adoption](docs/adoption.md#the-desktop-app-on-a-mac). On Windows,
`Verkstead-x86_64.msi` installs that same app into your own profile — no
administrator, its icon in the notification area, and `verkstead` on your
`PATH` for a terminal. It is unsigned there too, so SmartScreen stops the
install behind a **More info** link with **Run anyway** under it — also written
out beside the download in
[adoption](docs/adoption.md#the-desktop-app-on-windows). Which of the four you
want is [adoption](docs/adoption.md#getting-it-running).

**Sessions run on all three, and on all three behind a boundary.** One
description of what a session may reach, rendered over the mechanism each
platform has: bubblewrap on Linux, where the rest of the machine is not in the
session's namespace at all; Apple's sandbox on a Mac, where the machine is in
plain sight and refused; and an AppContainer on Windows, the platform's own
deny-by-default identity, where reach is an access-control entry written on
each real directory the description names. What is inside is the same on all
three, and [adoption](docs/adoption.md) says what a session can and cannot get
to on each. The daemon install is the NixOS module's.

## Where things are written down

**[Design](docs/design/verkstead.md)** — what Verkstead is and the decisions it
rests on, as settled in the planning session behind it.

**[MVP roadmap](docs/roadmaps/mvp/ROADMAP.md)** — the five stages from here to
a Verkstead that covers the whole loop, and the brief for each.

**[CONTEXT.md](CONTEXT.md)** — the project's vocabulary. Conversation, Brief,
Timeline, Question Set, Answer and the rest, defined once.

**[Adoption](docs/adoption.md)** — what Verkstead replaces, how to get it
running, and how a day's work goes through it from Brief to settled pull
request.

**[Development](docs/development.md)** — the dev shell, building the viewer,
and the loop for working on Verkstead itself.

**[Releasing](docs/releasing.md)** — how a tag would become the published
binaries, the AppImage, the dmg and the msi. Nothing has been released under
this name yet.
## Running it here (WSL, no nix)

`nix develop` is what [development.md](docs/development.md) assumes; there is no
nix on this box, so the system toolchain does it.

### Once

```bash
sudo apt install bubblewrap                                   # required — sessions
sudo apt install libgtk-3-dev libayatana-appindicator3-dev    # only for `desktop`
cargo install sccache                                         # optional — compile cache
```

Then add `/mnt/wsl` to `sandbox_binds` in the Data Directory's `config.yaml`,
or the settings page's **Paths**. Required: `/etc/resolv.conf` is a symlink into
`/mnt/wsl`, so a session without it has no DNS and every model call reports
`Request timed out`.

```yaml
sandbox_binds:
  - /mnt/wsl
```

### Build and run

```bash
(cd web && pnpm install && pnpm build)
cargo build -p verkstead-cli --no-default-features
./target/debug/verkstead serve --data-dir ~/verkstead-data
```

**`--data-dir` must be absolute.** It is taken verbatim, so a relative one ends
up in each Sandbox's own paths, where it means nothing: the session dies with
`bwrap: Can't chdir to ../verkstead-data/worktrees/<branch>: No such file or
directory` even though the Worktree is there.

Build the viewer **before** the first `cargo build` — the viewer is embedded
`allow_missing`, so a Rust build that runs first bakes in an empty one and the
workbench answers `503 the viewer was not built into this binary`.

`--no-default-features` drops the GTK tray. The Data Directory sits outside the
checkout because `secrets.yaml` is not in `.gitignore`.

### The link

```bash
echo "http://127.0.0.1:8422/?key=$(cat ~/verkstead-data/workbench.key)"
```

The startup log prints the same thing as `workbench=`. Every page is 401 without
the key; paste the link once and the browser keeps the cookie. Delete
`workbench.key` and the next start mints a new one.

### Stopping it

```bash
pkill -f 'verkstead[ ]serve'
```

Sessions are started `--die-with-parent`, so they go with the server; there is
no shutdown path and nothing is deleted. A stopped Conversation reads as stalled
and **Resume** restarts it. The `[ ]` stops the pattern matching its own shell.

### Rebuilding

| Changed | Command |
| --- | --- |
| `web/` | `(cd web && pnpm build)` — a running server picks it up, no restart |
| Rust | `cargo build -p verkstead-cli --no-default-features`, then restart |
| still `503` after a viewer build | `touch crates/server/src/viewer.rs` first — cargo does not watch `web/dist` |

### From there

Steps 3 to 6 of [development.md](docs/development.md#quickstart): add a repo,
**New conversation**, `ask` from a second terminal, answer in the browser.

## License

MIT — see [LICENSE](LICENSE).
