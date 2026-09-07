# 03. Remote access — the pane that reads

## What to build

A **Remote access** section on the settings page, in the card-and-pane shape
every other section there has, over a read that says what this machine's
Tailscale is actually doing: whether `tailscale` is on the machine at all,
whether the daemon it talks to is running, the node's own name, whether serve is
on, and the address that results.

Its word joins the settings' list of word-named openings, which is what gives a
section its route — the list is read by the opening a path is parsed into, by
the routes the page declares, and by the suite that walks them, so a word added
there arrives with the route that reaches it and cannot land on *No such page*.

**Three states, told apart**, because each has a different thing to say:

- no `tailscale` on the machine, which is an install pointer;
- the binary with no daemon running, which is a machine where Tailscale is
  installed and not up — its own answer rather than *serve is off*;
- up, which names the node.

Read defensively. Both commands exit non-zero with a message on standard error
when the daemon is down, and the JSON one of them prints is documented as
subject to change between releases — so a shape the read does not recognise is
*cannot tell*, never *off*. Tailscale is nothing this repository pins: it is
whatever the host has.

Nothing is pressed in this task. The switch, the QR and **Reset key** are the
tasks after it; what lands here is a pane that tells the truth about the
machine.

## Acceptance criteria

- [ ] A machine with no `tailscale` on its PATH reads as such, with the pointer
      at how to install it.
- [ ] A machine with the binary and no daemon running reads as its own third
      thing rather than as serve being off.
- [ ] With Tailscale up, the pane names the node, says whether serve is on, and
      shows the address where it is.
- [ ] The pane's own path reaches it, which the settings-routes suite settles by
      walking every word-named opening.
