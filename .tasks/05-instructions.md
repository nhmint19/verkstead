# 05. Instructions that match

## What to build

The dependency step's instructions still describe the old world: every Linux
tab's Claude row says `sudo npm install -g` and warns that the native
installer's `~/.local/bin` is somewhere no session looks. Rewrite them to the
new one.

On every Linux tab and on Windows, the Claude row **leads with the native
installer** — `curl -fsSL https://claude.ai/install.sh | bash` on Linux, the
PowerShell one on Windows — and says that `~/.local/bin` must be on the `PATH`
of the shell Verkstead is started from, then the server restarted, because
the `PATH` is read once at startup. The distribution's package or npm stays as
the alternative, its note reworded: it lands in a system directory and needs
nothing else. macOS keeps Homebrew first, and says why the Dock is different:
an app started there has launchd's `PATH` and never sees `~/.local/bin`, so
Homebrew's prefix is where a Mac session finds things.

The old caveat against the native installer goes from every tab. The restart
note that was Windows' own — a `PATH` that changed while Verkstead was running
is one it has not read — now holds on every platform, so say it once above the
rows. The prose landing line is gone, task 04 having replaced it with the
real list; what is left above the rows is the restart note.

## Acceptance criteria

- [ ] Every Linux tab and the Windows tab lead the Claude row with the native
      installer and the shell-`PATH`-then-restart note; the macOS tab leads
      with Homebrew and explains the Dock.
- [ ] No tab says `~/.local/bin` is somewhere a session cannot look.
- [ ] The restart note is drawn on every tab, once.
- [ ] The viewer's suite covers the leading instruction on each tab.
