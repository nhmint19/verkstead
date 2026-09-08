# 06. A steer into Wrapping reads the branch afresh

## What to build

The design prose says a steer into Wrapping reads the branch afresh, and the
resolve press is a press rather than a steer for exactly that reason. The
store puts the review back to waiting only when a steer leaves into Grilling,
so a Done Conversation steered into Wrapping runs no review at all: the review
that settled on the way to Done is still settled, and the wrap-up reads it as
over.

Make every steer into Wrapping, from whatever state, put the review back to
waiting in the same transaction as the move, so the watchers it starts
dispatch a review session under the Review Pairing. **Resolve conflicts** is
untouched: it deliberately leaves the review's settle standing, and stays a
press of its own. A *No review* Conversation is untouched too, its review
settling again the moment the wrap-up looks.

## Acceptance criteria

- [ ] A Done Conversation steered into Wrapping runs a review session.
- [ ] A halted wrap-up steered into Wrapping runs a review session, whether
      or not one had already settled this round.
- [ ] The resolve press on a conflicted Done Conversation still runs no review.
