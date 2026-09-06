# 02. One browse, opening at HOME

## What to build

The path browser collapses to the one unbounded scope it already had for the
fields the boundary said nothing about: every field browses anywhere the server
can read. The two-scope split goes from the wire — the browse endpoint takes a
path and nothing else — and with it the "outside the watched paths" outcome a
listing could answer with.

**An ask with no path answers the server's own `HOME`.** That is the whole of
what replaces the roots a bounded field used to open on. It is a starting point
rather than a boundary: the listing walks up out of it like any other directory,
so a repository or an account above `HOME` is still reachable by browsing or by
typing. Where the server's `HOME` cannot be read — unset, gone, not a directory
— the ask falls back to what it answers today with nothing to open on: the
filesystem root on a Unix, and the drive list on Windows.

In the workbench the field loses its scope entirely: no scope prop, no separate
read of the roots, and no ceiling stopping the way back out. The Repo form's
path field and the Agent Profile form's three account fields go from bounded to
unbounded, which is the whole of what changes for them.

The fixtures the web suite draws from are written by `cargo test` rather than by
hand, so they are regenerated rather than edited. One of them is the roots
listing a bounded field opened on; it either becomes the `HOME` listing an empty
field now gets, or goes, with the two web suites that import it following.

## Acceptance criteria

- [ ] A browse with no path lists the server's own `HOME`; one asked for a
      directory above `HOME` lists that, so nothing is out of reach.
- [ ] A server whose `HOME` cannot be read answers the topmost listing instead —
      `/` on a Unix, the drives on Windows.
- [ ] The Repo form's field and the Profile form's three account fields open at
      `HOME` and browse anywhere the server can read; no scope is sent on the
      wire, and neither `BrowseScope` nor the outside-the-boundary listing
      outcome is left in the generated types.
- [ ] `cargo test` regenerates the browsing fixtures and the web suite passes
      against them.
