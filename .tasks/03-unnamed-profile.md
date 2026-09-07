# 03. The unnamed Profile

## What to build

A Profile may have no name. A name is for telling two accounts of one harness
apart, which is rare, and the wizard's accounts step is about to make Profiles
nobody has typed a word for.

**The store's name column becomes nullable.** The profiles table is STRICT and
SQLite cannot drop a `NOT NULL` in place, so this is a one-time rewrite of the
kind the migrations module already carries several of: a new table beside the
old one, the rows copied across, the old one dropped and the new one renamed.
Written to be safe against a database that has already had it, decided by what
it rewrites rather than by a version kept somewhere.

**Uniqueness becomes two rules**, both left to the index to refuse rather than
looked up first:

- at most one unnamed Profile *per harness*;
- no two named Profiles alike.

**`Nameless` is retired.** A Profile with no name is legal now, so the refusal
goes from the server, the viewer's wording and their tests. What the form sends
for *no name* is the null rather than the empty string. The taken-name refusal
stays and gains its sibling: a second unnamed Profile for a harness that already
has one is refused by name.

**Every reading of a Profile's name** shows nothing where the harness mark and
the model already say enough, and **Default** where a name must be shown. Most
of this already exists — the viewer's reading helper drops a Profile's name
wherever it is the only one of its backend — so what this task settles is the
rest: the pairing rows and their pickers, the settings card, and the session
record.

**The session record** copies the Profile's name at launch, so that a Profile
renamed or deleted later does not take the answer with it. That column becomes
nullable too, through a rewrite beside the first. Its wire field is already
nullable, where null has meant *recorded before Verkstead wrote it down*; it now
also means an unnamed Profile, and both draw as nothing, so no reader changes
shape.

## Acceptance criteria

- [ ] Two unnamed Claude Profiles are refused, naming the rule; an unnamed
      Claude beside an unnamed Codex is saved; two Profiles named alike are
      still refused.
- [ ] A database written before this opens with its named Profiles intact and
      both rules enforced afterwards.
- [ ] An unnamed Profile shows no name in the pairing rows and pickers where the
      mark and the model say enough, reads **Default** where a name must be
      shown, and its sessions record and draw as the harness and model alone.
- [ ] The `Nameless` refusal is gone from the server, the viewer and their
      tests, and saving a Profile with no name goes through.
