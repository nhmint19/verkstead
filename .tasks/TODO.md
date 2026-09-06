# Open the boundary

The Watched Path leaves Verkstead. It was one rule — a Repo is registered only
from within one, an Agent Profile's account sits inside one, the path browser is
bounded by one — and it made the first run worse than the protection was worth:
it refused the account a Profile most naturally names, the human's own login
under `~`. What keeps a session to its own Conversation is the Sandbox, composed
from the Repo and the Profile that Conversation names, and none of that needs a
second fence around the first.

So a bare `verkstead serve --data-dir .` with no flags registers a git
repository root anywhere the server can read and saves an Agent Profile over
`~/.claude`; every path field browses anywhere and opens at the server's own
`HOME` rather than at `/`; the settings page's Paths section holds sandbox binds
and nothing else; and the NixOS module's `watchedPaths` becomes `paths`, the
directories bound read-write into the unit, with no Verkstead meaning and no
minimum. The security cost is named and accepted: the UI API is reachable from a
session over the loopback and was before — stage 02 is what closes it, and
nothing here compensates for it.

Roadmap stage: [01: Open the boundary](docs/roadmaps/onboarding/01-open-the-boundary.md)

## Tasks

- [x] 01: Register and save anywhere — [details](01-register-and-save-anywhere.md)
- [x] 02: One browse, opening at HOME — [details](02-one-browse-opening-at-home.md)
- [ ] 03: The flag, the setting and the Paths section — [details](03-the-flag-and-the-paths-section.md)
- [ ] 04: `paths` on the NixOS module — [details](04-paths-on-the-nixos-module.md)
- [ ] 05: Docs and vocabulary — [details](05-docs-and-vocabulary.md)
