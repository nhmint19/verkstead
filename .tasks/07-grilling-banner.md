# 07. The banner

## What to build

The dismissable banner on the Conversation page, above the Timeline, that points
at the Remote access pane — and the server-side flag that ends it everywhere.

**When it is drawn**: a Conversation that is Grilling, with a session running and
no Question Set on its Timeline yet. That is the window while the first Set is
being *prepared* — the first moment the human has nothing to do at the desk —
and it closes the moment a Set is there. A Set waiting is something to do rather
than a moment to be told about answering from a phone.

It comes back at the next grilling start, and the one after, until it is
dismissed.

**The dismissal is kept on the server**, beside the archived switch's — a
one-row table read back on every load — so one press on any device ends it on
all of them. Not a browser's own storage: the whole point of the banner is a
human standing at a desk who is about to pick up a phone, and a dismissal that
did not travel would meet them again on the device it was pointing them at.

## Acceptance criteria

- [ ] The banner is drawn above the Timeline on a Conversation that is grilling
      with a session running and no Question Set yet, and it leads to the Remote
      access pane.
- [ ] It goes as soon as the first Question Set lands, and comes back at the
      next grilling start until it is dismissed.
- [ ] Dismissed on one device, it is absent on another after a reload.
- [ ] Nothing else on the page moves when it is not drawn.
