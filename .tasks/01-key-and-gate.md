# 01. The key and the gate

## What to build

A Verkstead that answers **401 to anything that has not shown the Workbench
Key**. One long-lived random secret, made at first start and kept in the Data
Directory — which no Sandbox mounts — in a file of its own at mode `0600`,
*beside* the settings files rather than inside one of them: clearing the GitHub
token writes `secrets.yaml` empty, so a key kept there would be destroyed by an
ordinary settings save.

The link that hands the key over is any address with `?key=…`. The server takes
it, sets the cookie, and redirects to the same path with the query stripped, so
the secret never stays in a URL bar, a history entry or a referrer. A cookie
carrying the current key is the whole of being logged in; a wrong one is no
better than none.

**What is gated**: the viewer's own JSON namespace, and the SPA fallback that
answers every page of the workbench. **What stays open**:

- `/api/v1/health`, which is not a question about anybody's work.
- The Conversation-scoped session API, which is a session's own and scoped
  already.
- The PWA's three static files — the service worker, the web manifest and the
  icons. They carry nothing about anybody's work, and a browser fetches a
  manifest without credentials unless the document's link tag says otherwise, so
  gating them costs installability for nothing.

The Share Viewer needs no exemption at all: it is a self-contained file
published to a secret gist and reads nothing of the API.

**Two attachments, not one.** The viewer's routes are merged into the router in
one place and the fallback is added in another, so the gate goes on twice. And
make the gate something a router is *given*: the existing constructors the
suites are built on stay open — several hundred requests across them assume it —
while the router the binary serves is keyed, and the suites that ask about the
gate itself get a keyed constructor of their own.

## Acceptance criteria

- [ ] A request carrying no cookie gets 401 from the viewer's own namespace and
      from a workbench page, and 200 from `/api/v1/health` and from a session's
      own Conversation-scoped API.
- [ ] Opening any path with `?key=` sets the cookie and redirects to that path
      without the query; a request carrying a wrong key stays 401.
- [ ] The service worker, the web manifest and the icons answer 200 with no
      cookie, so the viewer is still installable.
- [ ] `curl` run inside a Conversation's Sandbox gets 401 from the viewer's
      namespace — asked the way the Linux sandbox suite asks everything, by
      running a probe inside rather than by reading the flags it was built with.
- [ ] The key file is `0600`, survives clearing the GitHub token through the
      settings, and a second start reads the key the first made rather than
      issuing another.
