# Wrap up a pull request

A pull request that was not drafted through Verkstead — opened by hand, by a
contributor, by the old tools — can be taken into the pipeline at its wrap-up:
a Conversation is started on it, and the ordinary Wrapping loop takes it from
there. The review under the Review Pairing reads the branch and what has been
said on it, *No review* skips that, red checks are fixed, comments answered,
mergeable waited on, and the Conversation settles to Done. Nothing about that
loop knows or cares who opened the pull request; what was missing was a door
into it.

Settled by the grilling: the compose page's dropdown becomes **Other actions**,
one nested level per action in the shared menu's one card — **Continue a
roadmap**, holding what adopting draws today, and **Wrap up a pull request**.
It is drawn whenever the box is empty and nothing is loaded, and a level with
nothing under it reads disabled rather than vanishing. *Adopt* stays the name
in the code and the vocabulary, widened to pull requests; what the human reads
is the friendlier word. The open pull requests are read off GitHub through the
configured `gh` for every registered Repo with a GitHub remote, when the
compose page opens and again on each reopen: any author, forks left out, one
a Conversation already holds — Closed included — listed with its row leading
to that Conversation. Picking a free one locks a card over the box, prefills
the box with the title and description as an editable Brief, disables the Repo
picker, and hides the branch, the base and the grilling picker. Both presses
create a Draft that records the pull request it holds. Taking it up fetches,
makes the head branch off origin or fast-forwards a local one that is behind,
refuses one that is ahead or diverged or checked out elsewhere, checks the
worktree and every companion out, records the head as the base commit with
GitHub's base branch beside it so the Timeline draws only what Verkstead adds,
and records the pull request as the move from Draft into Wrapping. In passing,
every steer into Wrapping puts the review back to waiting, as the design prose
already says it does.

## Tasks

- [x] 01: Other actions, and a roadmap is continued — [details](01-other-actions.md)
- [x] 02: The open pull requests — [details](02-open-pull-requests.md)
- [x] 03: Loading a pull request into the composer — [details](03-loading-a-pull-request.md)
- [x] 04: Taking it up — [details](04-taking-it-up.md)
- [x] 05: The wrap-up over a taken-up pull request — [details](05-the-wrap-up-over-it.md)
- [ ] 06: A steer into Wrapping reads the branch afresh — [details](06-steer-reads-afresh.md)
