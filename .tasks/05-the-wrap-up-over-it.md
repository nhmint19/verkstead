# 05. The wrap-up over a taken-up pull request

## What to build

Prove the ordinary wrap-up over a Conversation that arrived in Wrapping by
take-up rather than by its own finish step, and close whatever gaps that
proves.

The review session runs under the Review Pairing, in the worktree, with the
edited Brief in its prompt and no handoff, and with everything already said
on the pull request folded into what it reads, exactly as a wrap-up's first
review is. Under *No review* the review settles at once and what stands on the
pull request is batched to a responding session, as today. Red checks are
fixed on their two attempts, a conflict has its session, and the Conversation
reaches Done with nothing outstanding, sharing to the pull request where that
switch is on. A review that splits a backlog out sends the Conversation back
to be built, and the finish wraps it up again on the pull request it already
had.

A server coming back up over a taken-up Conversation carries on wrapping it —
its watchers restart from the record — and a Resume, a steer into Wrapping and
a steer into Follow-up all work from it, the record holding a pull request in
the Conversation's own Repo. Where any of that reads something a take-up never
wrote — a handoff, a direction, a commit on the branch beyond the base — it is
made to read the absence as the ordinary case.

## Acceptance criteria

- [ ] A review session dispatched over a taken-up Conversation is started on
      the edited Brief and told what was said on the pull request, and a
      *No review* take-up dispatches a responding session over the same.
- [ ] A restart over a taken-up Conversation resumes its watchers, and a steer
      into Follow-up and back is accepted.
- [ ] With the review settled, the checks green, nothing said and the pull
      request mergeable, the Conversation settles to Done and leaves the
      share comment where the switch is on.
