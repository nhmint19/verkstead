# Onboarding roadmap

A fresh Verkstead sets itself up: the server enters **onboarding mode** at
startup when its objective is unmet, and a wizard at `/setup` walks the human
through dependencies, accounts and git before landing them on a compose page
that can make a repository. Two decisions underneath it landed with it: the
**Watched Path** goes, and the workbench gets a **key** a session cannot hold.
The decisions and their why are in
[ADR-0015](../../adr/0015-open-boundary-and-workbench-key.md) and
[ADR-0016](../../adr/0016-onboarding.md); the terms are in
[CONTEXT.md](../../../CONTEXT.md), which each stage updates as it retires or
adds them.

Each stage is one feature: one branch, one review unit. Task chunkings inside
the briefs are provisional — re-grounded against the codebase when the stage
starts.

Stage 01 shapes 03 and 04: the wizard has no paths step and detects accounts
in HOME, and Create repo's parent is any directory. Stage 02 shapes 03 only in
that `/setup` sits behind the key like every page. Stages 03 and 04 do not
depend on each other and could swap; the order here is the one picked.

## Stages

- [x] 01: Open the boundary — [brief](01-open-the-boundary.md)
- [x] 02: The workbench key, and Remote access — [brief](02-workbench-key-and-remote-access.md)
- [x] 03: The onboarding wizard — [brief](03-onboarding-wizard.md)
- [x] 04: The zero-state compose page and the repo modal — [brief](04-zero-state-and-repo-modal.md)
