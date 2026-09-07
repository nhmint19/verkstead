# 05. Create on GitHub too

## What to build

The tick in the Create modal, and the half of the create that reaches GitHub.

**Drawn only where a token is saved.** The settings the page already reads say
whether there is one without ever handing it over, which is the whole of what
this needs to know. Where there is one the tick is drawn and starts on, and the
repository it makes is private: a repository made from here is somebody's work
before it is anybody else's business, and public is a decision to take
deliberately rather than by leaving a box alone.

**Where there is no token the tick is not drawn at all**, and the modal says a
remote is needed before the work is finished — the pipeline ends in a push and a
pull request, so a repository with nowhere to push is one that will stop halfway
through the first Conversation. A sentence rather than a refusal: the local
repository is still worth making, and the token can be saved afterwards.

**On the server it is `gh repo create` with the name, private, sourced from the
directory, `origin` added and pushed** — run after the initial commit, because
there is nothing to push before it. Authenticated as the configured token the way
the host's `gh` already is: the token through the environment, read at the moment
of the call rather than held from startup, so one saved or rotated through the
settings page reaches the next call without a restart.

**A GitHub failure after the local repository exists is not a failed create.**
The directory, the commit and the registration all stand, so the answer carries
the Repo *and* what failed rather than choosing between them, the modal says what
happened, and the Repo lands on the draft exactly as a clean create's does.

## Acceptance criteria

- [ ] With a token saved and the tick left on, the created repository is private
      on GitHub with `origin` set and `main` pushed.
- [ ] A GitHub failure after the local repository exists leaves it registered and
      picked, and the modal says what failed.
- [ ] With no token saved the tick is not drawn, and the modal says a remote is
      needed before the work is finished.
