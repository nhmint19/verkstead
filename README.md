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

My own notes for this machine, where [development.md](docs/development.md)'s
`nix develop` has nothing to enter — there is no nix on this box. The system
toolchain covers it: `cargo`, `node`, `pnpm`, `gcc`, `git` and `gh` are all on
`PATH` already, and nothing in the server half wants a system library — rustls
rather than OpenSSL, and SQLite bundled.

### Once

Two packages are missing, and neither of them is needed to bring the workbench
up:

```console
$ sudo apt install libgtk-3-dev libayatana-appindicator3-dev   # for `desktop`
$ sudo apt install bubblewrap                                  # to start sessions
```

GTK is what `crates/cli`'s default-on `desktop` feature links, so without it,
build with `--no-default-features` — the same headless build the musl CLI and
the nix package take, and the tray icon is the only thing it costs. Without
`bwrap` the server and the workbench come up as normal and a session refuses to
start: it is the whole of the Linux Sandbox's mechanism.

`sccache` is worth a third line, and is the one that is purely speed. The dev
shell carries one; without it the server says so at startup and turns compile
caching off, so every session compiles its dependencies again from source.
Crate *downloads* are still shared either way.

```console
$ cargo install sccache        # or: sudo apt install sccache
```

### The one bind WSL makes necessary

**Without it a session has no DNS, and every model call fails.** A Sandbox is a
mount namespace: `/etc` is bound into it read-only and the network is shared,
which on an ordinary Linux box is the whole of what resolving a name takes. On
WSL it is not, because `/etc/resolv.conf` here is a *symlink* to
`/mnt/wsl/resolv.conf` — and `/mnt/wsl` is not bound, so the link dangles and
the file is simply not there inside.

What that looks like is not a DNS error. `claude` inside the session reports it
as `API error · Retrying in 1s`, then `Request timed out. · attempt 3/10`, and
climbs to 10 attempts before giving up; the `Remote managed settings failed to
load` banner in the same status line has the same cause.

Add `/mnt/wsl` to the **Sandbox Configuration** — the settings page's **Paths**
section, or `sandbox_binds` in the Data Directory's `config.yaml`:

```yaml
sandbox_binds:
  - /mnt/wsl
```

It is read afresh every time a Sandbox is composed, so the running server picks
it up with no restart — but a session already running was composed without it
and has to be stopped and resumed. To check it from a shell, the probe is the
sandbox itself:

```console
$ bwrap --unshare-all --share-net --ro-bind /usr /usr --ro-bind /bin /bin \
    --ro-bind /lib /lib --ro-bind /lib64 /lib64 --ro-bind /etc /etc \
    --ro-bind /mnt/wsl /mnt/wsl --proc /proc --dev /dev \
    /bin/bash -c 'getent hosts api.anthropic.com'
```

Drop the `/mnt/wsl` line from that and it fails, which is the difference.

### Every time

```console
$ (cd web && pnpm install && pnpm build)
$ cargo run -p verkstead-cli --no-default-features -- serve --data-dir ../verkstead-data
```

**The order of those two matters, and only the first time.** `crates/server`
embeds `web/dist` with `#[allow_missing = true]` — the attribute that lets a
checkout which has never built the viewer still compile — so a `cargo build`
run while `web/dist` is absent bakes in an *empty* viewer, permanently. The
server then comes up, answers the agents' API, accepts the Workbench Key, and
answers the workbench itself with `503 the viewer was not built into this
binary: run pnpm build in web/`. Building the viewer afterwards does not lift
it: the Rust side has to be compiled again, and cargo does not watch that
folder, so it needs pushing —

```console
$ touch crates/server/src/viewer.rs && cargo build -p verkstead-cli --no-default-features
```

Once `web/dist` existed at compile time, the usual rule applies and the viewer
build is only for a `web/` that has changed: `rust-embed`'s `debug-embed` is
deliberately off, so a debug `serve` reads the folder off disk per request and
a rebuilt viewer is visible to a server already running, without a recompile.

The first `cargo build` is a cold build of the whole workspace — about three
minutes here — and the ones after it are seconds.

**The way in is the `workbench=` link in the startup log** — the address with
the **Workbench Key** on it. Every page answers 401 without the key, and
pasting that link once leaves the cookie in the browser. The key is
`workbench.key` in the Data Directory, made at the first start and read back at
every one after it.

### Stopping it

`kill` is the whole of it, and it is the designed answer rather than a blunt
one. There is no shutdown path in the server at all —
`crates/server/src/sandbox/outliving.rs` says so outright, and the reason is
`--die-with-parent`: every session's `bwrap` is started with it, so the kernel
ends them the moment the server goes. The tray app's **Exit** is the same stop.

```console
$ pkill -f 'verkstead serve'
$ pgrep -af 'verkstead serve|bwrap'   # silent means everything went
$ ss -ltnp | grep 8422                # and nothing is on the port
```

Ctrl-C in the terminal it was started in does the same. A session that was
mid-run is ended where it stands rather than asked to finish: its Conversation
stays at whatever the database last recorded and reads as stalled when the
server comes back, and **Resume** is what restarts it. A `verkstead ask`
holding a long-poll dies with the server and exits nonzero.

Nothing is deleted by stopping — the next start reads the same database, the
same Worktrees and the same Workbench Key. A second Verkstead started while the
first is still listening does not race it: it says so and exits nonzero.

### Why the data directory is outside the checkout

`--data-dir .` is what development.md says, and it puts `config.yaml`,
`secrets.yaml`, `workbench.key`, `worktrees/`, `handoffs/` and `skills/` in the
repository root. Only `*.db` is in `.gitignore` — so a `secrets.yaml` holding a
GitHub token shows up as untracked in every `git status` here, one `git add .`
away from being committed. A sibling directory is just as deletable alongside
the checkout and has none of that; omitting the flag altogether uses
`~/.local/share/verkstead`, which is where an installed Verkstead keeps it.

Two files in there are worth writing before the first Conversation — both can
be saved from the settings page instead:

```yaml
# secrets.yaml
github_token: ghp_...
```

```yaml
# config.yaml
git_author:
  name: ...
  email: ...
```

Every session started after that gets the token as `GH_TOKEN` and git
configured through the environment. With neither, sessions still start: `gh`
inside says it is not logged in, and git asks to be told who you are.

### From there

Steps 3 to 6 of [development.md](docs/development.md#quickstart) are the loop
itself — add a repo, **New conversation**, then `ask` from a second terminal
and answer in the browser.

## License

MIT — see [LICENSE](LICENSE).
