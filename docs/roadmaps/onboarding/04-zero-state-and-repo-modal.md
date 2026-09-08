# 04. The zero-state compose page and the repo modal

## Goal

A Verkstead with nothing to list is a compose page that can make a
repository. Demonstrable: with no unarchived Conversation, `/` lands on a
sidebar-less compose page headed by the wordmark and the gear; the Repo
dropdown's **Create repo** makes a directory with a `README.md` commit on
`main`, optionally on GitHub too, and the draft is on it; **Open repo**
registers an existing repository the same way; archiving the last
Conversation puts the page into that state and the pinned archived switch
brings the sidebar back.

## Decisions in force

All from [ADR-0016](../../adr/0016-onboarding.md); what bears on this stage:

- **The zero state is a fact about the list**, not about onboarding: it
  applies whenever the sidebar's list *as filtered* is empty — no unarchived
  Conversation, and archived ones absent or hidden. Switching *Show archived*
  on with archived Conversations present brings the sidebar back with them
  in it.
- **What it draws.** `/` redirects to `/compose`. The compose page stands on
  the frame with no `conversations` pane — the shape the share page already
  uses. The wordmark (the sidebar's `PaneHead` heading class) takes the pane
  title's place with the gear after it; the archived switch is pinned to the
  page's bottom-left and hidden when nothing is archived. The settings page
  keeps its sidebar.
- **Two rows at the foot of the Repo dropdown**, behind a rule, in both
  `RepoSelect` and `RepoChoice` — one control, so a draft's composer gets
  them too. The `Listbox` has no action rows today; add them as a row kind
  rather than a second control.
- **Create repo** is a modal on the existing `Modal`: a parent directory
  (`PathField` browsing anywhere, remembering the last parent on the device
  and opening at the server's `HOME` where there is none, which on a first run
  there never is) and a name. The server makes the directory, `git init -b
  main`, writes a `README.md` holding the name, commits it as the configured
  `git_author` the way `publishing.rs` commits, registers it, and answers with
  the `RepoView`. Refusals are named outcomes in a 200 body like
  registration's: the parent missing, the directory existing, a name git will
  not take, no author configured, `git` failing.
- **Create on GitHub too**: drawn only when a token is saved, ticked,
  private, `gh repo create <name> --private --source . --remote origin
  --push` after the initial commit, authenticated as the token the way the
  host `gh` already is. Without a token the modal says a remote is needed
  before the work is finished. A GitHub failure after the local repository
  exists leaves the local one registered and says what failed.
- **Open repo** is the registration form's path field in a modal — `scope`
  gone with stage 01, `repositories` marked — with the same refusals drawn
  in the modal.
- **Either way the Repo becomes the draft's**: the compose state's repo id,
  or a move on a saved draft, exactly as picking a registered one.

## Proposed tasks (provisional)

1. **The zero-state frame.** The redirect from `/`, the compose page without
   a sidebar, the wordmark and gear in the head, the pinned archived switch,
   the return of the sidebar when the filtered list is non-empty.
   - Archiving the last Conversation on one device moves another to the zero
     state on its next read.
2. **Create repo on the server.** The endpoint, the directory and the
   commit, the registration, the refusals, the GitHub half.
   - A created repo has one commit on `main` by the configured author and
     is registered under its directory name.
3. **The dropdown rows and the two modals.** Action rows in the `Listbox`;
   the Create modal with its tick; the Open modal over the registration
   field; the pick landing on the draft.
   - A refused create leaves the modal open with the reason under the field.
4. **Docs and vocabulary.** CONTEXT.md's **Brief** and **Repo** entries gain
   the zero state and the two rows; the design doc's UI section records the
   zero state as settled.

## Re-verify at start

- Stage 01 landed: `PathField` has no `scope`, registration admits any root,
  and a browse with no path opens at the server's `HOME`.
- Stage 03 landed if it went first: the wizard ends on `/compose`, so the
  finish lands in this page's zero state.
- `Panes.tsx` still stands two panes when `conversations` is absent.
- `RepoSelect` / `RepoChoice` in `web/src/workbench/Setup.tsx` are still the
  two places the dropdown is drawn, and `Listbox` still has no row kinds.
- `publishing.rs`'s commit-as-author pattern and `github.rs`'s
  token-authenticated `gh` are still the shapes to reuse.
- `git init -b` needs git 2.28; the floor the AppImage and the dmg assume.
