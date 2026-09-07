//! What the wizard is drawn from: whether this Verkstead can do anything yet,
//! what machine it is standing on, and what is missing from it.
//!
//! Nothing here is a setting. Every field is what the server found a moment
//! ago — a `PATH` walked, a `bwrap` run, `/etc/os-release` read — which is why
//! there is no stored half to reconcile against: an install that lands while
//! the page is open reads here on the next re-read, exactly as one that was
//! there all along.
//!
//! **Except the one field that is not read at all.** [`OnboardingView::mode`]
//! is the verdict the server reached at startup and has held ever since, and it
//! is what says whether the wizard is the only page there is. The steps beside
//! it are read afresh like everything else, so the two can disagree — a Profile
//! deleted while the server runs is a step that stands unmet under a mode that
//! is off — and that disagreement is the decision rather than a gap in it: the
//! wizard is a first run rather than a state to fall back into, and what says
//! so is the settings page's own empty state.
//!
//! **The distro is one of eight**, and they are the wizard's tabs: the two
//! platforms that have no distro to speak of, the five Linux distributions
//! whose install commands are written down, and everything else. The detected
//! one opens and the other seven stay reachable, because a detection read off
//! `ID_LIKE` is a guess about a derivative and the human is the one looking at
//! the machine.
//!
//! **The accounts are the machine's too.** What is found in the server's own
//! home is one account per harness at most — each shape is a fixed path under a
//! home — offered as the Profile it would be saved as, with whether that
//! harness is on the machine carried beside it. See [`AccountView`].
//!
//! **A row is present, absent or neither.** Neither is the Windows sandbox
//! row, which is not a thing to install there — see [`DependencyState`] — and
//! an absent one carries whatever the machine said about it, which on Linux is
//! the failed `bwrap` run's own standard error and nowhere else anything at
//! all.

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// Whether a fresh Verkstead can do anything yet, and what it would take.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS), ts(export_to = "types.ts"))]
pub struct OnboardingView {
    /// Whether onboarding mode is on: the verdict of the objective, reached
    /// once at startup and standing until the wizard finishes.
    pub mode: bool,

    /// Whose rules this machine plays by, which is what the sandbox row and the
    /// wording around it are about.
    pub platform: Platform,

    /// And which install commands it takes, which is the tab that opens.
    pub distro: Distro,

    /// Every row of the dependencies step, in the order it is drawn.
    pub dependencies: Vec<DependencyView>,

    /// And every agent account already on this machine, in the order the
    /// harnesses above are drawn. Empty on a machine that has none, which is
    /// the step saying what to run rather than what to tick.
    pub accounts: Vec<AccountView>,

    /// And whether each of the three steps stands met, at this moment.
    pub steps: StepsView,
}

/// The three platforms, as the viewer receives one.
///
/// Its own type beside [`Distro`], which carries the same fact for two of its
/// eight values: the distro is which set of commands to draw, and this is which
/// machine they are for — a sandbox row that ticks, one that is run, and one
/// that is nothing to install.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS), ts(export_to = "types.ts"))]
pub enum Platform {
    Linux,
    MacOs,
    Windows,
}

/// And which of the wizard's eight tabs this machine is.
///
/// The five Linux distributions whose commands are written down, everything
/// else that is a Linux, and the two platforms whose answer is the platform's
/// own. Read off `/etc/os-release` — `ID` first and then `ID_LIKE`, so that a
/// derivative gets its parent's commands rather than the generic list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS), ts(export_to = "types.ts"))]
pub enum Distro {
    MacOs,
    Windows,
    NixOs,
    Ubuntu,
    Fedora,
    Debian,
    Arch,

    /// A Linux naming none of them, which gets the generic list of what is
    /// needed rather than a command that would be wrong.
    OtherLinux,
}

/// One row of the dependencies step: a thing a session needs, and whether this
/// machine has it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS), ts(export_to = "types.ts"))]
pub struct DependencyView {
    pub dependency: Dependency,
    pub state: DependencyState,
}

/// What a row is about.
///
/// Flat rather than a harness variant carrying an [`crate::AgentType`]: the
/// step draws seven rows with an instruction apiece, and which of them are
/// harnesses is a fact about the objective rather than about the drawing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS), ts(export_to = "types.ts"))]
pub enum Dependency {
    /// What a session is run inside: `bwrap` on Linux, the seatbelt on macOS,
    /// and nothing at all on Windows.
    Sandbox,

    /// Which is not optional: every Conversation is a branch and a worktree.
    Git,

    Claude,
    Codex,
    Grok,
    OpenCode,

    /// The one row that never gates, GitHub being a choice rather than a
    /// dependency — see the objective in ADR-0016.
    Gh,
}

/// And whether the machine has it.
///
/// Flat on the wire — `{"state": "Absent", "trouble": "…"}` — so the viewer
/// narrows on a field rather than unwrapping a variant name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state")]
#[cfg_attr(feature = "typescript", derive(TS), ts(export_to = "types.ts"))]
pub enum DependencyState {
    /// A session would find it.
    Present,

    /// It would not.
    Absent {
        /// What the machine said about it, where anything was said at all: the
        /// standard error of a `bwrap` that is installed and would not run,
        /// which is where an unprivileged user namespace that is switched off
        /// says so in its own words. Nothing where the answer was simply that
        /// no such program is on the sandbox's `PATH`.
        trouble: Option<String>,
    },

    /// It is not a thing on this platform: the Windows sandbox row, where a
    /// session's boundary is an identity rather than something to install.
    NotApplicable,
}

/// One account this machine already has, offered as the Agent Profile it would
/// be saved as.
///
/// **Found rather than configured.** The server looks in its own home for the
/// shapes a session mounts an account from, so what is here is an account some
/// agent wrote there by being logged into once. At most one per harness, each
/// shape being a fixed path under a home — which is the same fact an unnamed
/// Profile's uniqueness is per harness for.
///
/// **And it is the whole account**, in the shape the profile form sends one:
/// the wizard saves a ticked row by handing it straight back to the profile
/// create, with no name and the models this build knows for its harness, rather
/// than by naming paths of its own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS), ts(export_to = "types.ts"))]
pub struct AccountView {
    /// What was found, ready to be saved as it stands.
    pub account: crate::ProfileAccount,

    /// And whether the harness that runs it is on this machine — the same
    /// answer that harness's [`Dependency`] row carries, so a row is offered
    /// ticked or drawn greyed without the viewer pairing the two lists up. An
    /// account whose binary is missing is not one to make a Profile of yet.
    pub harness: bool,
}

/// Whether each of the wizard's three steps stands met, read at the moment the
/// endpoint is asked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS), ts(export_to = "types.ts"))]
pub struct StepsView {
    /// A sandbox, `git`, and at least one of the four harnesses.
    pub dependencies: bool,

    /// At least one Agent Profile, however it was made.
    pub accounts: bool,

    /// And a git author: both halves of one, because that is what git asks for.
    /// The GitHub token is not in this — see ADR-0016.
    pub git: bool,
}
