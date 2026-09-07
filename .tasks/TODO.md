# The onboarding wizard

A fresh Verkstead sets itself up. At startup the server works out whether its
objective is met — a sandbox, `git` and at least one of the four harnesses
present, at least one **Agent Profile**, and a git author — and enters
**onboarding mode** where it is not. While that mode is on every URL is
`/setup`: a wizard that names each missing dependency with this distro's own
install command and ticks it within ten seconds of the install landing, offers
the agent accounts it found in the server's home as Profiles, and asks for the
git author. The last Continue clears the mode and lands on the compose page.

The same start on a machine that has everything, one Profile and an author
opens the workbench as it always did. There is no skip and no re-entry: the
verdict is reached once, at startup, so deleting the last Profile mid-run is
the settings page's empty state to say rather than a first-run page over work
somebody has.

Roadmap stage: [03: The onboarding wizard](docs/roadmaps/onboarding/03-onboarding-wizard.md)

## Tasks

- [x] 01: The verdict and the endpoint — [details](01-verdict-and-endpoint.md)
- [ ] 02: The page and the gate — [details](02-page-and-gate.md)
- [ ] 03: The unnamed Profile — [details](03-unnamed-profile.md)
- [ ] 04: The dependencies step — [details](04-dependencies-step.md)
- [ ] 05: The accounts step — [details](05-accounts-step.md)
- [ ] 06: The git step and the finish — [details](06-git-step-and-finish.md)
- [ ] 07: Docs and vocabulary — [details](07-docs-and-vocabulary.md)
