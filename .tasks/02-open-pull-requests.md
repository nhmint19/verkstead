# 02. The open pull requests

## What to build

The second level of Other actions: **Wrap up a pull request**, listing every
open pull request across the registered Repos.

The server reads them through the configured `gh`, one `gh pr list` per
registered Repo that has a GitHub remote, run in that Repo's own path the way
the wrap-up's own reads are. One endpoint answers the whole list, grouped by
Repo, and each row carries the number, the title, the URL, the head branch,
the base branch and the author, plus which Conversation already holds it — a
pull request on any Conversation's record in that Repo, Done and Closed
included. A pull request whose head branch is in another repository (a fork)
cannot be pushed to over origin and is left out. A Repo with no GitHub remote,
or a `gh` that is absent, not logged in or will not answer, contributes no
rows and no error — what Verkstead does not know is not an empty list, but it
is not a broken page either.

The compose page reads the list when it opens and again on each reopen, every
Repo in parallel, and does not hold it on the device. Until the reading is
back the level opens to a loading row; once it is back empty the level reads
disabled, as the roadmap level does. One row per pull request, saying the
repo, the number and title, the head branch and the base it goes into, and the
author. A row for a pull request a Conversation already holds says so and
goes to that Conversation's page rather than loading anything; a free row does
nothing yet — loading it is task 03.

## Acceptance criteria

- [ ] With a stubbed `gh` answering two open pull requests, one of them from a
      fork, the level draws one row naming its repo, number, title, head, base
      and author.
- [ ] A pull request on a Closed Conversation's record draws as held and its
      press navigates to that Conversation.
- [ ] A Repo without a GitHub remote and one whose `gh` will not answer add
      no rows, and the level reads disabled once every reading is back empty.
