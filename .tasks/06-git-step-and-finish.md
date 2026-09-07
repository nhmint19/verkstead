# 06. The git step and the finish

## What to build

The wizard's last step, and the end of onboarding mode.

**What git needs**: an author name and an email, both required, and the GitHub
token, optional — GitHub may not be in use at all.

**Prefilled from what the server can see, and saved by nobody but the human.**
The settings module's rule is *told, not found*, and a prefill somebody confirms
is still telling. Name and email come from the server's own `git config
--global`; the token comes from `GH_TOKEN`, then `GITHUB_TOKEN`, in the server's
environment, and then from the host `gh`'s own login. **Each field is labelled
with where its value was found**, a field the server found nothing for is empty,
and **nothing is written until Continue**.

**The save is the settings save the page already has**, verification included —
one save, one place that knows how to write those two files. The GitHub login is
what the verification answers with and is shown rather than typed: a token that
authenticates as the wrong account is the mistake this catches. A token that
fails verification says so with everything the human typed still in front of
them.

**The last Continue clears the mode for this run** and the app lands on
`/compose`. Every URL is reachable again from that moment. The mode never comes
back until the next start, and a start with the objective met opens the
workbench exactly as it always did. Stage 04 makes the compose page's zero
state; until it lands, the compose page is the one that exists.

## Acceptance criteria

- [ ] The three prefills arrive labelled with their sources — the global git
      config, the environment variable that held the token, or the host `gh` —
      and a value the server found nowhere leaves its field empty.
- [ ] Nothing is written until Continue, which saves through the whole-settings
      save; the step then shows the login the verification returned, and a token
      that fails verification says so without losing what was typed.
- [ ] The last Continue clears the mode, the app lands on `/compose`, and every
      other URL stops redirecting.
- [ ] A restart on a machine that now meets the objective opens the workbench
      with no wizard; a restart that does not meet it opens `/setup` again.
