# 07. The note goes, and the docs say what is true

## What to build

Windows has a boundary now, so the note that said it had none goes — and with it
the whole idea of an unsandboxed session, which no platform can produce any more:
Linux without `bwrap` refuses a session, a Mac has `sandbox-exec`, and Windows
has a container. The server-decided value would be false everywhere forever, and
a note nothing can ever draw is a note nobody keeps.

So the value, the sentence in all three of its places — above **Start work** on
the composer, beside the terminal on the session pane, and on the Conversation
Terminal's pane — the component that draws it, the view's field, the fixtures
that carry it and the tests that assert it all go together. The Windows sessions
suite's own test that a Conversation there says its sessions are not sandboxed
goes with them.

And then the documentation says what a Windows session can and cannot reach, in
the places a reader looks:

- **`docs/adoption.md`**'s Windows section reads beside the Mac's — what is
  inside, what is refused, and **the entries left on real directories**, which is
  the one thing about this boundary that has no equivalent on either other
  platform and that a human should not have to discover. That they are per
  Conversation, that closing takes them away, and that a crash is swept up at the
  next startup.
- **`CONTEXT.md`**'s **Sandbox** term names three renderings beside bubblewrap's
  and the seatbelt's, and loses stage 01's paragraph beginning *On Windows a
  session has none of this yet*. The **Terminal** term loses its clause about no
  Sandbox until that stage lands.
- **`README.md`**'s *sessions run on all three* paragraph stops saying two of
  them are behind a boundary.
- **`docs/design/verkstead.md`**'s revision note from stage 01 gets its own
  revision: the AppContainer arrived and the note went.

## Acceptance criteria

- [ ] The viewer draws no note on any Windows view, and vitest no longer has a
      case that it does; nothing in the API type or the fixtures still carries
      the value.
- [ ] `docs/adoption.md`'s Windows section reads beside the Mac's and says what
      the entries on real directories are, when they are written and when they go.
- [ ] `CONTEXT.md`'s Sandbox term names three renderings and carries no interim
      state, and its Terminal term no longer says a Windows shell is unsandboxed.
