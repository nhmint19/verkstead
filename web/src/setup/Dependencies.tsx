//! The wizard's first step: what a session needs, what this machine is missing,
//! and the exact command that fixes it here.
//!
//! **The rows are the server's**, in the order it drew them — the sandbox,
//! `git`, the four harnesses and `gh` — and so is every state on them. What this
//! side adds is the words: what each row is for, why three of the seven hold
//! nothing up, and the instruction under the ones that are not there yet. A row
//! that is present says so and nothing else; there is nothing to do about it.
//!
//! **Which instruction is a tab rather than a fact.** The detected OS opens and
//! the other seven are a press away — see
//! [`./instructions.ts`](./instructions.ts) — because the detection comes off
//! `/etc/os-release` and a derivative names its parent. Nothing about the rows
//! changes with the tab: the tab is which machine the commands are *for*, and
//! whether a program is there is a fact about the machine the server is on.
//!
//! **Three of the rows never gate.** The objective is a sandbox, `git` and at
//! least *one* of the four harnesses, so three unticked harnesses hold nothing
//! up, and `gh` holds nothing up at all — GitHub being a choice rather than a
//! dependency. What decides whether Continue is pressable is the server's own
//! verdict on this step (`steps.dependencies`) rather than a second reading of
//! the rows here, which would be the objective written down twice.
//!
//! **And Continue is a press.** The step never advances under somebody's hands,
//! however the rows tick while they are watching: an install that lands makes
//! the button pressable and nothing else. The page's own interval is what makes
//! that happen within ten seconds of the install — see
//! [`./SetupPage.tsx`](./SetupPage.tsx).

import { For, Show, createSignal, type JSX } from "solid-js";

import { Copy } from "../Copy";
import { HarnessMark } from "../HarnessMark";
import { AGENT_NAME, type AgentType } from "../agents";
import type {
  Dependency,
  DependencyState,
  DependencyView,
  Distro,
  OnboardingView,
} from "../api/types";
import { Note } from "../notices";
import { Mark, type Standing } from "./Mark";
import { DISTROS, GUIDES, type Instruction } from "./instructions";
import styles from "./Dependencies.module.css";

/// What each row is called where somebody reads it.
///
/// The harnesses are [`AGENT_NAME`]'s, which is what they are called everywhere
/// else in the app: a row saying *Claude* beside a Profile list saying *Claude
/// Code* would be two names for one backend.
const NAMES: Record<Dependency, string> = {
  Sandbox: "A sandbox",
  Git: "git",
  Claude: AGENT_NAME.Claude,
  Codex: AGENT_NAME.Codex,
  Grok: AGENT_NAME.Grok,
  OpenCode: AGENT_NAME.OpenCode,
  Gh: "gh",
};

/// And what each is for, which is also which of them hold the step up.
const WHY: Record<Dependency, string> = {
  Sandbox: "Every session runs inside one",
  Git: "Every Conversation is a branch and a worktree",
  Claude: "One harness is enough",
  Codex: "One harness is enough",
  Grok: "One harness is enough",
  OpenCode: "One harness is enough",
  Gh: "Optional: pull requests and reviews",
};

/// And what a row reads instead, where this platform has no such thing to have:
/// the Windows sandbox row, whose whole answer is the note under it.
const MOOT = "Not applicable";

/// Which rows are a harness, so that the row wears the same mark the rest of the
/// app draws that backend with.
const HARNESSES: Partial<Record<Dependency, AgentType>> = {
  Claude: "Claude",
  Codex: "Codex",
  Grok: "Grok",
  OpenCode: "OpenCode",
};

/// The step, over the reading the page last had.
export function Dependencies(props: {
  /// How this machine stands, as the server last said.
  reading: OnboardingView;

  /// And the step after this one, opened by Continue.
  onwards: () => void;
}): JSX.Element {
  // Which OS the commands are for. The detected one to begin with, and the
  // human's after they have picked: a machine whose `ID_LIKE` sent them to the
  // wrong tab is exactly who this control is for, and the reading re-arriving
  // every ten seconds must not put them back.
  const [showing, setShowing] = createSignal<Distro>(props.reading.distro);

  const guide = () => GUIDES[showing()];

  /// Whether this step stands met, which is the server's own verdict on it.
  const met = () => props.reading.steps.dependencies;

  return (
    <div class={styles.step}>
      <p class={styles.standing}>
        A session runs inside a sandbox, works in a git worktree, and is one of
        four agents. What is missing is what the commands below install.
      </p>

      <div
        class={styles.tabs}
        role="group"
        aria-label="What these commands are for"
      >
        <For each={DISTROS}>
          {(distro) => (
            <button
              type="button"
              class={styles.tab}
              data-distro={distro}
              aria-pressed={showing() === distro}
              onClick={() => setShowing(distro)}
            >
              {GUIDES[distro].title}
            </button>
          )}
        </For>
      </div>

      <Note>{guide().landing}</Note>

      <ul class={styles.rows}>
        <For each={props.reading.dependencies}>
          {(row) => <Row row={row} instruction={guide().rows[row.dependency]} />}
        </For>
      </ul>

      <div class={styles.onwards}>
        <button
          type="button"
          class={styles.continue}
          disabled={!met()}
          onClick={() => props.onwards()}
        >
          Continue
        </button>

        <Show when={!met()}>
          <Note>
            A sandbox, git and one of the four agents are what a session cannot
            start without. This page reads the machine again every ten seconds,
            so an install that lands ticks its row here.
          </Note>
        </Show>
      </div>
    </div>
  );
}

/// One row: what it is, whether this machine has it, and — where it does not —
/// what to run.
function Row(props: {
  row: DependencyView;
  instruction: Instruction;
}): JSX.Element {
  const state = (): DependencyState => props.row.state;

  /// What a failed run said, which is only ever the Linux sandbox row's: a
  /// `bwrap` that is installed and will not make a namespace says why in its
  /// own words, and no sentence written here would be as useful.
  const trouble = (): string | null => {
    const said = state();

    return said.state === "Absent" ? said.trouble : null;
  };

  return (
    <li
      class={styles.row}
      data-dependency={props.row.dependency}
      data-state={state().state}
    >
      <div class={styles.head}>
        <Mark standing={standing(state())} />
        <span class={styles.name}>
          <HarnessMark
            of={HARNESSES[props.row.dependency] ?? null}
            class={styles.harness}
          />
          {NAMES[props.row.dependency]}
        </span>
        <span class={styles.why}>
          {state().state === "NotApplicable" ? MOOT : WHY[props.row.dependency]}
        </span>
      </div>

      <Show when={trouble()}>
        {(said) => <pre class={styles.trouble}>{said()}</pre>}
      </Show>

      {/* Nothing under a row that is already there: what is left to say about a
          program a session would find is nothing. */}
      <Show when={state().state !== "Present"}>
        <Instructed of={props.instruction} />
      </Show>
    </li>
  );
}

/// What to run, where to get it, and what neither says for itself.
function Instructed(props: { of: Instruction }): JSX.Element {
  return (
    <div class={styles.instruction}>
      <Show when={props.of.command}>
        {(command) => (
          <div class={styles.command}>
            <pre>{command()}</pre>
            {/* Somebody is going to paste this into a terminal, and a phone
                cannot select a line of it usefully. */}
            <Copy of={command()} class={styles.copy} />
          </div>
        )}
      </Show>

      <Show when={props.of.link}>
        {(href) => (
          <p class={styles.vendor}>
            <a href={href()} target="_blank" rel="noreferrer">
              {href()}
            </a>
          </p>
        )}
      </Show>

      <Show when={props.of.note}>{(note) => <Note>{note()}</Note>}</Show>
    </div>
  );
}

/// How a row's state stands, as the mark beside its name.
function standing(state: DependencyState): Standing {
  switch (state.state) {
    case "Present":
      return "met";
    case "NotApplicable":
      return "moot";
    default:
      return "waiting";
  }
}
