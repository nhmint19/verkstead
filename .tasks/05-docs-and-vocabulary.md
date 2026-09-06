# 05. Docs and vocabulary

## What to build

The prose catches up with the code. Nothing here changes behaviour; what it
changes is every place a reader would still be told about a rule that no longer
exists.

**CONTEXT.md.** The **Watched Path** entry is retired outright rather than
rewritten. **Repo** loses the clause saying it is registered from inside one,
and every other term that leans on the boundary loses the sentence that does —
the account an Agent Profile names, the Data Directory's contrast against one,
and the Sandbox Configuration's *configured where the Watched Paths are*, which
is now the installation and the workbench and nothing else.

**`adoption.md`.** The NixOS block says `paths`, described as the directories
bound read-write into the unit with no Verkstead meaning, no default and no
minimum. This is also where the thing a refusal no longer says gets said: a
repository or an account the unit was not told to bind is answered *missing*,
and `paths` is where to add it. Keep the note that `home` is only what `HOME`
means for the service, and add that an account under it a session must write is
named in `paths` too.

**`development.md`.** The quickstart runs the server with no boundary flag, the
`config.yaml` and settings-JSON examples lose their watched paths, and the
walkthrough adds a repo from anywhere rather than from inside one.

**The design doc's product decisions.** The bullet describing the two-sided
boundary, the sandbox-configuration bullet that says it is configured where the
watched paths are, the Paths card bullet — which now counts binds alone and sits
below the Repos — and the two path-selector bullets, which describe one scope
opening at the server's `HOME` rather than two.

**The settings module's own doc comment**, whose `config.yaml` example still
lists a key that no longer parses.

The ADRs and this roadmap's briefs are the record of the decision and keep
saying what was decided; they are not swept.

## Acceptance criteria

- [ ] No document under `docs/` names a Watched Path except the ADRs and this
      roadmap's own briefs.
- [ ] CONTEXT.md has no **Watched Path** entry, and no other term is left
      leaning on one.
- [ ] `adoption.md` documents `paths` as the unit's read-write binds, says the
      `home`-plus-`paths` case, and says that a path the unit was not told about
      is answered *missing* and where to add it.
- [ ] The design doc describes one browse scope opening at `HOME`, and a Paths
      card counting binds below the Repos.
