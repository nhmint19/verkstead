# 01. The zero-state frame

## What to build

What Verkstead is when there is nothing to list.

**The zero state is a fact about the list rather than about onboarding.** It
holds whenever the sidebar's list *as filtered* is empty — no unarchived
Conversation, and the archived ones either absent or hidden — and it stops
holding the moment something is in that list, whether it arrived by being created
or by *Show archived* being switched on. So it is decided where the list is read
rather than in the route table: a redirect written into the routes would be a
second opinion about a list only the sidebar reads.

**The server has to say whether anything is archived at all.** The sidebar's list
is filtered in SQL by the switch's own setting, so an empty list cannot be told
apart from a list with archived rows hidden behind it — and that is exactly what
decides whether the pinned switch is drawn. The endpoint that answers whether the
archived are being shown answers whether there are any as well, so the page reads
one fact rather than two.

**What it draws.** `/` redirects to `/compose`, replacing rather than pushing so
that Back does not walk into a page that will only redirect again. The compose
page stands on the frame with no `conversations` pane — the two-pane shape the
share page already uses, which the frame already knows how to widen. The wordmark
takes the pane title's place with the gear after it: both live inside the
conversations sidebar today and are private to it, so they are lifted to
somewhere both pages can draw them rather than written a second time. The
archived switch is pinned to the page's bottom-left, and is not drawn at all
where nothing is archived.

**The settings page keeps its sidebar**, which is the one page in the zero state
that does: it is a list of its own, and a settings page with no way back to the
conversations would be a page somebody had to type their way out of.

## Acceptance criteria

- [ ] With no unarchived Conversation, `/` lands on `/compose` drawn with no
      conversations pane, the wordmark and the gear in its head.
- [ ] Archiving the last Conversation moves another device to the zero state on
      its next read, and unarchiving it brings the sidebar back.
- [ ] Switching *Show archived* on with archived Conversations present brings the
      sidebar back with them in it; where nothing is archived the pinned switch
      is not drawn.
- [ ] The settings page keeps its sidebar while the zero state holds.
