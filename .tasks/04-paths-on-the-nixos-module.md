# 04. `paths` on the NixOS module

## What to build

The module's `watchedPaths` becomes **`paths`**, and it is re-meant rather than
renamed. It has no Verkstead meaning at all: it is the list of directories bound
read-write into the unit, repositories and accounts alike. `ProtectHome=tmpfs`
and `ProtectSystem=strict` hide everything the unit is not told to bind, so the
unit's namespace is still the module's to widen — but it is a namespace and not
a boundary, and the server is told nothing about it.

So there is no minimum and no assertion: a build naming none is a legal build,
and the option's own description carries the reasoning it used to borrow from
the boundary. The flag it used to be passed as does not exist any more after
task 03, so the list feeds the unit's binds and nothing else.

`home` stays bound read-only, which means an account under it that a session
must *write* has to be named in `paths` as well. Say so in the option's
description — it is the one composition somebody will get wrong.

A path the unit was never told to bind is simply not there as far as the server
is concerned, and the refusal it gets is *missing*, exactly like any path the
server cannot see. Nothing on the wire says which install this is and nothing
names the option: **where to add it is a thing the adoption documentation
explains** — task 05 — rather than something a refusal carries.

The VM test proves both halves against a unit that actually boots. A repository
under a directory `paths` names registers. A repository the unit was not told to
bind is answered *missing* — and the directory that case is built in matters:
the hardening leaves `/srv` visible even where it is not bound, so a repository
there would now register perfectly well. The case that is genuinely outside the
namespace is one under `/home`, which `ProtectHome=tmpfs` replaces wholesale and
the module binds back only what it was told to.

The module's evaluation check is what proves a build with no `paths` at all,
since it is the one that instantiates the module rather than booting it; it also
names the option today and follows the rename.

## Acceptance criteria

- [ ] The VM test registers a repository under a directory `paths` names and is
      answered `Added`, and is answered `Missing` for a repository under `/home`
      that `paths` does not name.
- [ ] `nix build .#checks.x86_64-linux.vm` passes.
- [ ] `nix flake check` passes, with the module's evaluation check putting a
      configuration with no `paths` at all through it.
- [ ] The option's description says that `home` is bound read-only and that an
      account under it a session must write is named in `paths` as well.
