# 01. Register and save anywhere

## What to build

Registering a Repo and saving an Agent Profile stop asking the boundary whether
the path is allowed. A git repository root anywhere the server can read is
registered; an account anywhere the server can read is saved; and the refusal
that said otherwise is gone from the wire, from the server and from the viewer's
message maps.

The boundary type itself stays standing for now — the path browser and the
settings page's resolution report are both still built on it, and they are tasks
02 and 03. What goes here is only its use as an *admission*: the arms that
answered "outside", and the readers that consulted it on the way to a
registration, a Profile save, a Profile listing and the `broken` check a stored
Profile gets when its account has moved.

The resolving those admissions did on the way has to stay, because it is not the
boundary's — a path is still resolved before anything is done with it, so that
`..` is taken out and symlinks are followed, and the resolved path is what is
stored. What was one call answering four things becomes the two questions that
are left: absolute, and there.

Refusals that stay, by name: a Repo path must be absolute, must be a repository
root, and that repository must be able to say its default branch; an account
must be absolute, be there, and be a directory (or, for Claude, a directory and
a config file) of its harness's shape. Only the `Outside` arms go — on
registration, on each of a Profile's path fields, and on the `Broken` a listed
Profile reports.

## Acceptance criteria

- [ ] A git repository root outside every directory the server was started with
      registers and comes back `Added`, and what is stored is the resolved path.
- [ ] A relative path, a directory that is not a repository root, a path with
      nothing at it, and a repository with no default branch are each still
      refused under their own name.
- [ ] An Agent Profile whose account is the server's own `~/.claude` pair saves,
      lists and reads back unbroken.
- [ ] No `OutsideWatchedPaths` remains in `Registered`, `ProfileSaved` or
      `Broken`, in the generated TypeScript, or in the viewer's message maps.
