# The zero-state compose page and the repo modal

A Verkstead with nothing to list is a compose page that can make a repository.
Whenever the sidebar's list *as filtered* is empty — no unarchived Conversation,
and archived ones either absent or hidden — the sidebar is not drawn, `/`
redirects to `/compose`, the wordmark and the gear take the pane title's place,
and the archived switch is pinned to the page's bottom-left. Switching it on with
archived Conversations present brings the sidebar back with them in it, because a
switch pinned to a page has to show something when it is pressed.

And the Repo dropdown grows two rows at its foot, behind a rule: **Create repo**
makes a directory with a `README.md` commit on `main` and, where a token is
saved, the same repository on GitHub; **Open repo** registers one that already
exists. Both are drawn in the one control, so a draft's composer gets them too,
and either way the Repo is registered and becomes the draft's.

Roadmap stage: [04: The zero-state compose page and the repo modal](docs/roadmaps/onboarding/04-zero-state-and-repo-modal.md)

## Tasks

- [x] 01: The zero-state frame — [details](01-zero-state-frame.md)
- [x] 02: The action rows, and Open repo — [details](02-action-rows-and-open-repo.md)
- [x] 03: Create repo on the server — [details](03-create-repo-on-the-server.md)
- [x] 04: The Create repo modal — [details](04-create-repo-modal.md)
- [ ] 05: Create on GitHub too — [details](05-create-on-github-too.md)
- [ ] 06: Docs and vocabulary — [details](06-docs-and-vocabulary.md)
