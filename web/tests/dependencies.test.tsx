//! The wizard's first step: the rows this machine is missing, the eight tabs of
//! instructions over them, and the press that is refused until a session could
//! start at all.
//!
//! Two halves, and they are asked about in two ways. **What is drawn** is put in
//! front of the component as a reading — the golden fixtures `cargo test` writes
//! out of the real endpoint, so what the step is drawn over is what the server
//! actually said. **What the instructions say** is a table of this side's own,
//! the same eight answers on every Verkstead, so it is read as the table it is:
//! a command that is wrong is wrong on every machine, and nothing has to be
//! mounted to catch it.
//!
//! Two of the readings here are made rather than served: a machine whose `bwrap`
//! will not run, and the two platforms this runner is not. Each is a fixture with
//! one field moved — see [`stating`] — because what the wizard has to draw for a
//! Mac is a fact about the page, while what the server says about a Mac is a
//! Rust unit test in `crates/server/src/onboarding.rs` and asked about there.

import { fireEvent, render, waitFor } from "@solidjs/testing-library";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type {
  Dependency,
  DependencyState,
  Distro,
  OnboardingView,
} from "../src/api/types";
import { Dependencies } from "../src/setup/Dependencies";
import { SetupPage } from "../src/setup/SetupPage";
import { DISTROS, GUIDES } from "../src/setup/instructions";
import { SETUP_STEP } from "../src/setup/steps";
import { json, serving } from "./serving";
import fresh from "./fixtures/onboarding-fresh.json" with { type: "json" };
import partWay from "./fixtures/onboarding-part-way.json" with { type: "json" };

/// A bare Ubuntu machine: nothing installed at all, and so no step met.
const FRESH = fresh as OnboardingView;

/// And the same machine once what a session needs is on it: a sandbox, `git`
/// and one harness, with the other three and `gh` still absent.
const PART_WAY = partWay as OnboardingView;

/// What a machine with unprivileged user namespaces switched off says when
/// `bwrap` is asked for a namespace — the line that names what to change, which
/// is why the row carries it rather than a sentence of the wizard's own.
const REFUSAL =
  "bwrap: No permissions to creating new namespace, likely because the kernel " +
  "does not allow non-privileged user namespaces";

/// The same reading with one row's state replaced.
///
/// The states this step has to draw outnumber the readings the server writes
/// fixtures for, and the missing ones are a machine's rather than a page's: a
/// `bwrap` that would not run, and the sandbox row on the two platforms this
/// runner is not. Every arm of the server's own answer is asked about in Rust,
/// where a platform is a value; what is asked here is what the page draws when
/// one arrives.
function stating(
  reading: OnboardingView,
  dependency: Dependency,
  state: DependencyState,
): OnboardingView {
  return {
    ...reading,
    dependencies: reading.dependencies.map((row) =>
      row.dependency === dependency ? { ...row, state } : row,
    ),
  };
}

/// A Mac, where the sandbox is Apple's own and there is nothing to install.
const A_MAC: OnboardingView = {
  ...stating(FRESH, "Sandbox", { state: "Present" }),
  platform: "MacOs",
  distro: "MacOs",
};

/// And a Windows machine, where a session's boundary is an identity rather than
/// a program.
const WINDOWS: OnboardingView = {
  ...stating(FRESH, "Sandbox", { state: "NotApplicable" }),
  platform: "Windows",
  distro: "Windows",
};

/// The step alone, over a machine that stands as `machine` says.
///
/// No query client and no router: the step is handed the reading the page
/// already has, and how often that reading is taken again is the frame's own —
/// see `onboarding.test.tsx`.
function mount(machine: OnboardingView) {
  const onwards = vi.fn();

  return {
    ...render(() => <Dependencies reading={machine} onwards={onwards} />),
    onwards,
  };
}

/// One row of the step.
function row(container: ParentNode, dependency: Dependency): HTMLElement {
  return container.querySelector<HTMLElement>(
    `[data-dependency="${dependency}"]`,
  )!;
}

/// Every tab, in the order they were drawn.
function tabs(container: ParentNode): HTMLElement[] {
  return [...container.querySelectorAll<HTMLElement>("[data-distro]")];
}

/// And which one is showing.
function showing(container: ParentNode): string | undefined {
  return container.querySelector<HTMLElement>(
    '[data-distro][aria-pressed="true"]',
  )?.dataset.distro;
}

/// The press onwards, whatever it is drawn as.
///
/// Found by its own words rather than by a role query, which is what every other
/// suite here does with a button that may not be there yet.
function onwards(container: ParentNode): HTMLButtonElement {
  return [...container.querySelectorAll("button")].find(
    (button) => button.textContent === "Continue",
  )!;
}

describe("the eight tabs of instructions", () => {
  it("opens the one this machine says it is, and draws all eight", () => {
    const { container } = mount(FRESH);

    expect(showing(container)).toBe("Ubuntu");
    expect(tabs(container).map((tab) => tab.dataset.distro)).toEqual([
      ...DISTROS,
    ]);
  });

  /// The detection is a guess off `/etc/os-release` — a derivative names its
  /// parent, and a machine nobody has heard of names nothing — so the other
  /// seven are a press away for the human who can see the machine.
  it("shows another machine's commands when its tab is pressed", () => {
    const { container } = mount(FRESH);

    expect(row(container, "Git").textContent).toContain("sudo apt install git");

    fireEvent.click(
      tabs(container).find((tab) => tab.dataset.distro === "Windows")!,
    );

    expect(showing(container)).toBe("Windows");
    expect(row(container, "Git").textContent).toContain(
      "winget install --id Git.Git",
    );
  });
});

describe("the rows", () => {
  /// A `bwrap` that is installed and will not make a namespace says why in its
  /// own words, and the line naming the sysctl to set is the one worth reading.
  it("puts the failed run's own words under the sandbox row", () => {
    const { container } = mount(
      stating(FRESH, "Sandbox", { state: "Absent", trouble: REFUSAL }),
    );

    expect(row(container, "Sandbox").querySelector("pre")!.textContent).toBe(
      REFUSAL,
    );
  });

  it("ticks the sandbox on a Mac, where it is Apple's own", () => {
    const { container } = mount(A_MAC);
    const sandbox = row(container, "Sandbox");

    expect(sandbox.dataset.state).toBe("Present");
    expect(sandbox.querySelector('[aria-label="done"]')).not.toBeNull();
    // Nothing under a row that is already there: there is nothing to do about
    // it, so there is no instruction to draw.
    expect(sandbox.textContent).not.toContain("brew install");
  });

  it("reads not applicable on Windows, in the wording the app already uses", () => {
    const { container } = mount(WINDOWS);
    const sandbox = row(container, "Sandbox");

    expect(sandbox.dataset.state).toBe("NotApplicable");
    expect(sandbox.textContent).toContain("Not applicable");
    expect(sandbox.textContent).toContain(
      "There is nothing to install: it is how the sandbox works on Windows.",
    );
  });

  /// GitHub is a choice rather than a dependency, so its row is drawn like any
  /// other and gates nothing: this machine is missing it and Continue is
  /// pressable all the same.
  it("draws gh with its instruction and holds nothing up", () => {
    const { container } = mount(PART_WAY);
    const gh = row(container, "Gh");

    expect(gh.dataset.state).toBe("Absent");
    expect(gh.textContent).toContain("sudo apt install gh");
    expect(onwards(container).disabled).toBe(false);

    // And a machine that has one draws the row ticked, with nothing left to
    // say about it.
    const has = mount(stating(PART_WAY, "Gh", { state: "Present" }));
    const ticked = row(has.container, "Gh");

    expect(ticked.querySelector('[aria-label="done"]')).not.toBeNull();
    expect(ticked.textContent).not.toContain("apt install");
  });

  /// Three harnesses left unticked hold nothing up either: a session runs under
  /// one Profile, and a Profile is of one agent type.
  it("draws the three harnesses this machine has not got and holds nothing up", () => {
    const { container } = mount(PART_WAY);

    expect(row(container, "Claude").dataset.state).toBe("Present");
    for (const harness of ["Codex", "Grok", "OpenCode"] as const) {
      expect(row(container, harness).dataset.state).toBe("Absent");
    }

    expect(onwards(container).disabled).toBe(false);
  });
});

describe("what each machine is told to run", () => {
  /// What each machine installs bubblewrap with — the five distributions whose
  /// package names are written down. *Other Linux* is not among them: what it
  /// gets is the generic list, because a command pasted from the wrong package
  /// manager is worse than a sentence.
  const BUBBLEWRAP: Partial<Record<Distro, string>> = {
    NixOs: "environment.systemPackages = [ pkgs.bubblewrap ];",
    Ubuntu: "sudo apt install bubblewrap",
    Fedora: "sudo dnf install bubblewrap",
    Debian: "sudo apt install bubblewrap",
    Arch: "sudo pacman -S bubblewrap",
  };

  /// And git, which every OS with a package manager has one for.
  const GIT: Partial<Record<Distro, string>> = {
    MacOs: "brew install git",
    Windows: "winget install --id Git.Git",
    NixOs: "environment.systemPackages = [ pkgs.git ];",
    Ubuntu: "sudo apt install git",
    Fedora: "sudo dnf install git",
    Debian: "sudo apt install git",
    Arch: "sudo pacman -S git",
  };

  it("names an exact command for bubblewrap on every distribution that has one", () => {
    for (const [distro, command] of Object.entries(BUBBLEWRAP)) {
      expect(GUIDES[distro as Distro].rows.Sandbox.command).toBe(command);
    }

    // And says what is needed in words where there is no command to give.
    expect(GUIDES.OtherLinux.rows.Sandbox.command).toBeUndefined();
    expect(GUIDES.OtherLinux.rows.Sandbox.note).toContain("bubblewrap");
  });

  it("names an exact command for git on every OS that has a package manager", () => {
    for (const [distro, command] of Object.entries(GIT)) {
      expect(GUIDES[distro as Distro].rows.Git.command).toBe(command);
    }

    expect(GUIDES.OtherLinux.rows.Git.note).toContain("git");
  });

  it("names an exact command for each harness the OS packages", () => {
    for (const distro of DISTROS) {
      for (const harness of ["Claude", "Codex", "OpenCode"] as const) {
        expect(GUIDES[distro].rows[harness].command).toBeTruthy();
      }
    }

    // The casks Homebrew keeps those two under, which a plain `brew install`
    // would not find.
    expect(GUIDES.MacOs.rows.Claude.command).toBe(
      "brew install --cask claude-code",
    );
    expect(GUIDES.MacOs.rows.Codex.command).toBe("brew install --cask codex");
    expect(GUIDES.MacOs.rows.OpenCode.command).toBe("brew install opencode");
  });

  /// The `grok-cli` in nixpkgs is somebody else's agent and the one on npm is a
  /// proxy around claude-code: either would install a different program under
  /// the name a session launches. So Grok Build is xAI's own page on all eight
  /// tabs and a command on none of them.
  it("gives Grok Build the vendor's page on every one of them", () => {
    for (const distro of DISTROS) {
      const grok = GUIDES[distro].rows.Grok;

      expect(grok.command).toBeUndefined();
      expect(grok.link).toBe("https://x.ai/cli");
    }
  });

  it("says where a binary has to land, and calls out Claude's own installer", () => {
    for (const distro of DISTROS) {
      expect(GUIDES[distro].landing).toBeTruthy();
      // The one installer that lands somewhere no session looks, said on the
      // row it belongs to rather than under the tab.
      expect(GUIDES[distro].rows.Claude.note).toContain("~/.local/bin");
    }
  });
});

describe("the press onwards", () => {
  it("is refused while what a session needs is missing", () => {
    const { container, onwards: pressed } = mount(FRESH);

    const button = onwards(container);
    expect(button.disabled).toBe(true);

    fireEvent.click(button);
    expect(pressed).not.toHaveBeenCalled();
  });

  it("opens the step after this one when it is pressed", () => {
    const { container, onwards: pressed } = mount(PART_WAY);

    fireEvent.click(onwards(container));
    expect(pressed).toHaveBeenCalledTimes(1);
  });
});

/// The whole page for this one, because what is being asked about is the install
/// landing in another window: the row that ticks is drawn by the step and the
/// re-read that ticks it is the frame's.
describe("an install that lands while the page is open", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
    localStorage.clear();
  });

  it("ticks the row within ten seconds and releases Continue, and advances only when pressed", async () => {
    // The bare machine first, and every read after it a machine somebody has
    // just installed what was missing on.
    serving(json(FRESH), json(PART_WAY));

    const client = new QueryClient({
      defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
    });
    const { container } = render(() => (
      <QueryClientProvider client={client}>
        <SetupPage />
      </QueryClientProvider>
    ));

    await waitFor(() => expect(row(container, "Sandbox")).toBeTruthy());
    expect(row(container, "Sandbox").dataset.state).toBe("Absent");
    expect(onwards(container).disabled).toBe(true);

    await vi.advanceTimersByTimeAsync(10_000);

    await waitFor(() =>
      expect(row(container, "Sandbox").dataset.state).toBe("Present"),
    );
    expect(onwards(container).disabled).toBe(false);

    // And nothing has moved: a step that advanced under somebody's hands is a
    // page that changed while they were reading it.
    expect(localStorage.getItem(SETUP_STEP)).toBeNull();
    expect(
      container.querySelector('[data-step="dependencies"]')!.getAttribute(
        "aria-current",
      ),
    ).toBe("step");

    fireEvent.click(onwards(container));

    await waitFor(() =>
      expect(localStorage.getItem(SETUP_STEP)).toBe("accounts"),
    );
  });
});
