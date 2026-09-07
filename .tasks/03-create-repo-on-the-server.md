# 03. Create repo on the server

## What to build

The endpoint that makes a repository, beside the one that registers one.

**What it is given** is a parent directory and a name. What it does is make the
directory under that parent, run `git init` onto `main`, write a `README.md`
holding the name, commit it, register the result, and answer with the same opened
Repo the registry's own pane is drawn from.

**The commit is why there is a README at all.** An empty repository has a default
branch and no commit, and a Conversation cannot take a base from one — so a fresh
repository needs something on `main` before it is worth registering, and a README
is what a fresh repository conventionally holds.

**It is committed as the configured git author**, said on the command line rather
than written into the new repository's config — the same shape the share
publishing uses, for the same reason: it is the same fact either way and one of
them leaves a file behind. Where no author is configured this refuses instead of
falling back the way publishing does. A share published as Verkstead is a share;
a repository whose first commit is by nobody is that repository's history.

**Every refusal is a named outcome in a 200 body**, the way registration's are,
because each is a different sentence to put in front of the human and none is
something to retry: the parent is not there, a directory of that name is there
already, the name is one git will not take, no author is configured, `git`
failed. A refusal registers nothing.

The filesystem half runs off the runtime the way registration's does — making a
directory and shelling out to git are both blocking, and a create is rare enough
that the thread it borrows costs nothing.

## Acceptance criteria

- [ ] A created repo has one commit on `main` by the configured author, holding a
      `README.md` naming it, and comes back registered under its directory name.
- [ ] Each of the five refusals — parent missing, directory existing, a name git
      will not take, no author configured, `git` failing — comes back named in a
      200 body rather than as a status.
- [ ] A failure after the directory exists is named as one, and nothing is left
      on the registry.
