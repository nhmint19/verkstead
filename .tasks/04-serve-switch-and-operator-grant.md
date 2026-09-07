# 04. The serve switch and the operator grant

## What to build

The switch on the Remote access pane that runs `tailscale serve --bg 8422` and
takes it off again, and what the machine does when it refuses.

**The position is read rather than remembered.** It comes off the serve state
the pane already reads, so a serve somebody set up by hand reads as on and the
switch turns *that* off.

**The operator grant.** `tailscale serve` from a process that is not root is
refused unless that user is Tailscale's operator. The daemon's answer is to show
the exact command for this machine's own user — `sudo tailscale set
--operator=<user>` — and to re-try on the next press: somebody who has run the
line in a terminal presses the switch again and it works. Nothing is escalated
here; the desktop app's own way of taking the grant is the task after this one.

**The NixOS module grants it itself**, wherever `services.tailscale.enable` is
on: the operator is set to the unit's own user. And `tailscale` joins what the
unit carries on its PATH — today that is bwrap, `gh` and sccache and nothing
else, so the daemon on that install cannot run the command at all.

The unit is heavily hardened, and whether it can reach the Tailscale daemon's
socket *through* that hardening is something to show rather than assume: the VM
test is where a relaxation the unit turns out to need would be caught, exactly
as it is for the sandbox.

## Acceptance criteria

- [ ] The switch on makes the address readable on the pane, and off takes it
      away; the position is read back from the serve state rather than held.
- [ ] A serve refused for want of the operator grant reads back the exact
      command for this machine's user, and a press after the grant re-tries and
      serves.
- [ ] `services.tailscale.enable` on a Verkstead host sets the operator to the
      unit's user, and `tailscale` is on the unit's PATH.
- [ ] The VM test shows the unit reaching the Tailscale daemon through its own
      hardening.
