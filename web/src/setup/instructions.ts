//! What to run on this machine to get what a session needs — one set of
//! instructions per operating system, and one instruction per row.
//!
//! **The commands are exact, or there is no command.** A row on a machine whose
//! package manager carries the program says the line to paste; a row on one that
//! does not says where the vendor's own installer is and nothing else. What is
//! never written here is a command that would install a *different* program
//! under the right name — see [`GROK_UNIX`], which is the whole of why Grok
//! Build is a link on all eight tabs.
//!
//! **Every instruction says where the binary has to land**, because installing
//! one is only half of it: a session resolves its programs on the sandbox's own
//! `PATH` (`crates/server/src/sandbox.rs`'s `LINUX_PATH` and `APPLE_PATH`)
//! rather than on the shell's, so an agent under `~/.local/bin` is a program
//! that runs perfectly in a terminal and is not there at all for a session.
//! Claude Code's own installer puts one exactly there, which is why the caveat
//! is on its row on every tab.
//!
//! **Eight tabs and not one**, because the detection is a guess. It comes off
//! `/etc/os-release`'s `ID` and then `ID_LIKE`, so a derivative names its parent
//! and something nobody has heard of names nothing: the detected tab opens and
//! the other seven stay a press away, for the machine the file was wrong about.
//!
//! Nothing here is on the wire. What the server says is what this machine *is*
//! and what it is missing; what to do about it is the same eight answers on
//! every Verkstead, so they are the viewer's own — see
//! `crates/render/src/onboarding.rs`, which carries the rows and the distro and
//! no prose at all.

import type { Dependency, Distro } from "../api/types";

/// One row's instruction, on one operating system.
///
/// A command, a link, or a note by itself — the sandbox row on macOS and on
/// Windows is nothing to install, and *other Linux* has the generic list of
/// what is needed where the five named distributions have a line to paste.
export type Instruction = {
  /// What to run, exactly, where this OS carries the program.
  command?: string;

  /// Or the vendor's own install page, where it does not.
  link?: string;

  /// And what the command does not say for itself: where the program has to
  /// land, what else it wants installed first, and what would put it somewhere
  /// no session can see.
  note?: string;
};

/// One operating system's answers: what the tab is called, where a session
/// looks on it, and an instruction for each of the seven rows.
export type Guide = {
  /// What the tab is called.
  title: string;

  /// Where a session's `PATH` goes on this machine, said once above the rows
  /// rather than seven times inside them.
  landing: string;

  /// And one instruction per row. Every row, on every OS: a tab with a gap in
  /// it is a row somebody is left staring at.
  rows: Record<Dependency, Instruction>;
};

/// The eight tabs, in the order they are drawn — the order
/// `verkstead_render::Distro` is written in, which is the two platforms, the
/// five distributions whose commands are written down, and everything else.
export const DISTROS: readonly Distro[] = [
  "MacOs",
  "Windows",
  "NixOs",
  "Ubuntu",
  "Fedora",
  "Debian",
  "Arch",
  "OtherLinux",
];

/// Where a session looks on a Linux machine, which is the machine's own
/// directories and not the shell's.
///
/// `LINUX_PATH` in `crates/server/src/sandbox.rs`, said in words. `~/.local/bin`
/// and `~/.nix-profile/bin` are the two an install is most likely to land in and
/// neither is on it, which is what the caveats below are about.
const LINUX_LANDS =
  "A session's PATH is /run/current-system/sw/bin, " +
  "/nix/var/nix/profiles/default/bin, /usr/local/bin, /usr/bin and /bin — the " +
  "machine's own directories rather than your shell's. A program installed " +
  "under your home directory runs in your terminal and is not there for a " +
  "session at all.";

/// And on a Mac, where Homebrew's own prefix is the first thing on it.
const APPLE_LANDS =
  "A session's PATH is /opt/homebrew/bin, /usr/local/bin, /usr/bin and /bin, " +
  "and nix's directories after them. Homebrew installs into the first of " +
  "those on Apple silicon and into /usr/local/bin on an Intel Mac, so " +
  "anything brew put there is somewhere a session looks.";

/// And on Windows, where there is no list to write down: a session gets the
/// `PATH` the server was started with.
const WINDOWS_LANDS =
  "A session on Windows runs on the PATH the server itself was started with, " +
  "so anywhere on the machine's PATH will do. A PATH that changed while " +
  "Verkstead was running is one it has not read: restart the server once the " +
  "install has landed.";

/// What Claude Code's own installer does, which is the one caveat that belongs
/// on a row rather than under the tab.
///
/// `~/.local/bin` is on nobody's sandbox `PATH` — see [`LINUX_LANDS`] — so the
/// native installer leaves a `claude` that works in a terminal and cannot be
/// launched by a session. Said on every tab, because the installer is the same
/// one everywhere and it is the way most people already have it.
const CLAUDE_ELSEWHERE =
  "Claude's own installer — curl -fsSL https://claude.ai/install.sh | bash — " +
  "puts the binary in ~/.local/bin, which is not on the PATH a session gets. " +
  "Install it the way above instead, or symlink it into /usr/local/bin.";

/// Grok Build, which is a link on every tab.
///
/// **The packages under that name are not xAI's.** The `grok-cli` in nixpkgs is
/// superagent-ai's agent and the `grok-cli` on npm is a proxy around claude-code;
/// either would install a different program under the name a session launches,
/// which is worse than having nothing to offer. So what is offered is the
/// vendor's own installer, and the caveat about where it puts the binary.
const GROK_UNIX: Instruction = {
  link: "https://x.ai/cli",
  note:
    "xAI's own installer puts grok in ~/.grok/bin and symlinks it into " +
    "/usr/local/bin where it can. That home directory is not on the PATH a " +
    "session gets, so check the symlink is there. The grok-cli packaged by " +
    "nixpkgs and the one on npm are other people's projects rather than xAI's " +
    "grok.",
};

/// The same on Windows, where the installer is the PowerShell one.
const GROK_WINDOWS: Instruction = {
  link: "https://x.ai/cli",
  note:
    "irm https://x.ai/cli/install.ps1 | iex is xAI's own installer. The " +
    "grok-cli on npm is somebody else's project rather than xAI's grok.",
};

/// A harness from npm, installed for the whole machine rather than for a user.
///
/// `-g` under the distribution's own node lands the binary in `/usr/local/bin`
/// or `/usr/bin` — both of them directories a session looks in — where an
/// npm prefix set to somewhere under `$HOME` would not.
function npm(pkg: string, node: string): Instruction {
  return {
    command: `sudo npm install -g ${pkg}`,
    note:
      "An -g install lands the binary in /usr/local/bin or /usr/bin, both of " +
      `which a session looks in. Where there is no npm yet: ${node}.`,
  };
}

/// And the same on Windows, which has no `sudo` and takes its node from winget.
function windowsNpm(pkg: string): Instruction {
  return {
    command: `npm install -g ${pkg}`,
    note:
      "Where there is no npm yet: winget install --id OpenJS.NodeJS. Restart " +
      "Verkstead after the first install, so that the server reads the PATH " +
      "npm was added to.",
  };
}

/// The same instruction with one more thing said under it.
///
/// What Claude Code's row has everywhere: where npm or brew puts the binary is
/// as true as it was, and [`CLAUDE_ELSEWHERE`] is the sentence about the
/// installer most people already used. Two notes rather than one replacing the
/// other, because dropping the first would take the landing with it.
function caveat(instruction: Instruction, said: string): Instruction {
  return {
    ...instruction,
    note: instruction.note ? `${instruction.note} ${said}` : said,
  };
}

/// Everything a NixOS machine installs, which is a line in the system
/// configuration rather than a command.
function nixos(attribute: string, note?: string): Instruction {
  return {
    command: `environment.systemPackages = [ pkgs.${attribute} ];`,
    note:
      (note ? `${note} ` : "") +
      "In configuration.nix, then sudo nixos-rebuild switch. A nix profile " +
      "install goes to ~/.nix-profile/bin, which is not on the PATH a session " +
      "gets.",
  };
}

/// The eight tabs' own answers.
///
/// Written out one tab at a time rather than composed out of a package manager
/// and a table of names: what the caveat under a row says is as much of the
/// instruction as the command is, and half of them are about the one machine
/// they are on.
export const GUIDES: Record<Distro, Guide> = {
  MacOs: {
    title: "macOS",
    landing: APPLE_LANDS,
    rows: {
      Sandbox: {
        note:
          "Apple's own sandbox-exec is on every Mac, and it is what a session " +
          "runs inside here. There is nothing to install.",
      },
      Git: {
        command: "brew install git",
        note:
          "Xcode's command line tools carry a git as well — xcode-select " +
          "--install — and either of the two is somewhere a session looks.",
      },
      Claude: caveat(
        {
          command: "brew install --cask claude-code",
          note: "A cask rather than a formula.",
        },
        CLAUDE_ELSEWHERE,
      ),
      Codex: {
        command: "brew install --cask codex",
        note: "A cask rather than a formula.",
      },
      Grok: GROK_UNIX,
      OpenCode: { command: "brew install opencode" },
      Gh: { command: "brew install gh" },
    },
  },

  Windows: {
    title: "Windows",
    landing: WINDOWS_LANDS,
    rows: {
      Sandbox: {
        note:
          "Sessions on this machine run inside an AppContainer, under an " +
          "identity Verkstead makes for each Conversation. There is nothing " +
          "to install: it is how the sandbox works on Windows.",
      },
      Git: { command: "winget install --id Git.Git" },
      Claude: caveat(windowsNpm("@anthropic-ai/claude-code"), CLAUDE_ELSEWHERE),
      Codex: windowsNpm("@openai/codex"),
      Grok: GROK_WINDOWS,
      OpenCode: windowsNpm("opencode-ai"),
      Gh: { command: "winget install --id GitHub.cli" },
    },
  },

  NixOs: {
    title: "NixOS",
    landing: LINUX_LANDS,
    rows: {
      Sandbox: nixos(
        "bubblewrap",
        "With Verkstead's own NixOS module, bwrap is on the service's PATH " +
          "already and there is nothing to add.",
      ),
      Git: nixos("git"),
      Claude: caveat(nixos("claude-code"), CLAUDE_ELSEWHERE),
      Codex: nixos("codex"),
      Grok: GROK_UNIX,
      OpenCode: nixos("opencode"),
      Gh: nixos("gh"),
    },
  },

  Ubuntu: {
    title: "Ubuntu",
    landing: LINUX_LANDS,
    rows: {
      Sandbox: { command: "sudo apt install bubblewrap" },
      Git: { command: "sudo apt install git" },
      Claude: caveat(
        npm("@anthropic-ai/claude-code", "sudo apt install nodejs npm"),
        CLAUDE_ELSEWHERE,
      ),
      Codex: npm("@openai/codex", "sudo apt install nodejs npm"),
      Grok: GROK_UNIX,
      OpenCode: npm("opencode-ai", "sudo apt install nodejs npm"),
      Gh: { command: "sudo apt install gh" },
    },
  },

  Fedora: {
    title: "Fedora",
    landing: LINUX_LANDS,
    rows: {
      Sandbox: { command: "sudo dnf install bubblewrap" },
      Git: { command: "sudo dnf install git" },
      Claude: caveat(
        npm("@anthropic-ai/claude-code", "sudo dnf install nodejs npm"),
        CLAUDE_ELSEWHERE,
      ),
      Codex: npm("@openai/codex", "sudo dnf install nodejs npm"),
      Grok: GROK_UNIX,
      OpenCode: npm("opencode-ai", "sudo dnf install nodejs npm"),
      Gh: { command: "sudo dnf install gh" },
    },
  },

  Debian: {
    title: "Debian",
    landing: LINUX_LANDS,
    rows: {
      Sandbox: { command: "sudo apt install bubblewrap" },
      Git: { command: "sudo apt install git" },
      Claude: caveat(
        npm("@anthropic-ai/claude-code", "sudo apt install nodejs npm"),
        CLAUDE_ELSEWHERE,
      ),
      Codex: npm("@openai/codex", "sudo apt install nodejs npm"),
      Grok: GROK_UNIX,
      OpenCode: npm("opencode-ai", "sudo apt install nodejs npm"),
      Gh: { command: "sudo apt install gh" },
    },
  },

  Arch: {
    title: "Arch",
    landing: LINUX_LANDS,
    rows: {
      Sandbox: { command: "sudo pacman -S bubblewrap" },
      Git: { command: "sudo pacman -S git" },
      Claude: caveat(
        npm("@anthropic-ai/claude-code", "sudo pacman -S npm"),
        CLAUDE_ELSEWHERE,
      ),
      Codex: npm("@openai/codex", "sudo pacman -S npm"),
      Grok: GROK_UNIX,
      OpenCode: npm("opencode-ai", "sudo pacman -S npm"),
      Gh: { command: "sudo pacman -S github-cli" },
    },
  },

  /// A Linux naming none of the five: what is needed, in words, rather than a
  /// command that would be wrong on the machine it was pasted into.
  OtherLinux: {
    title: "Other Linux",
    landing: LINUX_LANDS,
    rows: {
      Sandbox: {
        note:
          "Install your distribution's bubblewrap package — the program is " +
          "bwrap — and allow unprivileged user namespaces, which some kernels " +
          "ship switched off.",
      },
      Git: { note: "Install your distribution's git package." },
      Claude: caveat(
        npm(
          "@anthropic-ai/claude-code",
          "install your distribution's nodejs and npm",
        ),
        CLAUDE_ELSEWHERE,
      ),
      Codex: npm("@openai/codex", "install your distribution's nodejs and npm"),
      Grok: GROK_UNIX,
      OpenCode: npm("opencode-ai", "install your distribution's nodejs and npm"),
      Gh: {
        note:
          "Install your distribution's GitHub CLI package, which is gh or " +
          "github-cli.",
      },
    },
  },
};
