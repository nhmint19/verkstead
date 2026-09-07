//! Remote access, on the settings page: whether this machine can be reached
//! from a phone, and what stands between it and being.
//!
//! The Workbench Key is what makes this a section rather than a line in the
//! adoption docs (ADR-0015). A phone cannot reach the workbench until it holds
//! the key, and the address it would hold one against only exists once
//! `tailscale serve` is putting this machine's tailnet name in front of the
//! port Verkstead is listening on — which was a command somebody ran by hand.
//!
//! **Everything here is read rather than configured.** There is no saved half:
//! what the card and the pane draw is what two commands said a moment ago, so a
//! tailnet joined in a terminal and a serve set up by hand read here exactly as
//! ones set up from this page would. Which is why this section reads nothing of
//! the settings query every other section on the page shares — there is nothing
//! of it in either file.
//!
//! **Four things to say, because each wants something different done about it.**
//! No `tailscale` at all is an install, and nothing on this page can do it. The
//! binary with no daemon answering is a `tailscale up`, said in the machine's
//! own words because the line it prints names the service to start. Up is the
//! node's name and whether the workbench is served to the tailnet. And an
//! answer this build could not read is none of the three: Tailscale is whatever
//! the host has, so a shape nobody here has seen says so rather than being read
//! as the nearest state with room for it.
//!
//! **And one thing to press**, which is the serve itself. The switch's position
//! is the reading rather than anything this page remembers, so a serve somebody
//! set up in a terminal reads as on and the switch turns *that* off; a press
//! answers with the machine read again, and the switch settles wherever the
//! machine actually ended up. It is drawn only where the serve state was
//! readable, because a switch over *cannot tell* would be offering to turn on
//! something that may already be on.
//!
//! **The operator grant is a sentence, not a button.** Tailscale refuses a
//! serve from a process that is neither root nor the tailnet's operator, and the
//! server has no privilege to raise. So a refused press draws the line that
//! lifts it — for this machine's own user, as it is to be typed — and the next
//! press is the re-try.
//!
//! **And the address by itself lets nobody in.** The workbench answers 401
//! without the Workbench Key, so what a phone is actually pointed at is the
//! login link — the address with the key on it — which is drawn here as a QR
//! code and offered to copy beside it. The code is drawn in the browser from an
//! encoder the viewer ships: a workbench standing behind a secret has no
//! business handing that secret to a third party to render, and an install on a
//! tailnet may have nowhere to fetch from — see [`Qr`].
//!
//! **Reset key sits under them**, because it is what takes that link back. It
//! re-issues the secret, which logs every other device out: the QR and the link
//! redraw on the new one out of the answer, and whatever was holding the old one
//! meets a 401 on its next request. The browser that pressed it stays in — a
//! reset made from the phone on the tailnet is a reset made from the only device
//! that could reach this server at all.
//!
//! Nothing about it is confirmed twice. This is the human's own machine and the
//! sentence beside the press is what says what it costs, the way the sandbox
//! binds say what widening one costs: a press somebody has to acknowledge twice
//! is one they stop reading.

import { useMutation, useQueryClient } from "@tanstack/solid-query";
import { Match, Show, Switch as Choose, type JSX } from "solid-js";

import { CardButton } from "../CardButton";
import { Copy } from "../Copy";
import { PaneSticky } from "../Panes";
import { Switch } from "../Switch";
import { loadRemote, pressServe, resetKey } from "../api/client";
import type { RemoteView, ServePress, ServeView } from "../api/types";
import { useReading } from "../freshness";
import { Empty, ErrorLine, Note } from "../notices";
import { PaneHead } from "../workbench/PaneHead";
import { Qr } from "./Qr";
import styles from "./Remote.module.css";

/// Where a machine with no Tailscale on it gets one. The one thing this section
/// has to say that nothing on this page can do.
const DOWNLOAD = "https://tailscale.com/download";

/// The reading narrowed to one of its states, or `null` where it is in another.
///
/// Written once each rather than inline at every `when`, because that is what a
/// `Match` takes: a condition that is also the value the arm is drawn from, so
/// that the arm has the state's own fields in hand without asking again.
const absent = (told: RemoteView) => told.tailscale === "Absent";
const down = (told: RemoteView) => (told.tailscale === "Down" ? told : null);
const unreadable = (told: RemoteView) =>
  told.tailscale === "Unreadable" ? told : null;
const up = (told: RemoteView) => (told.tailscale === "Up" ? told : null);

/// And the serve state the same way.
const serving = (serve: ServeView) => (serve.serve === "On" ? serve : null);
const unread = (serve: ServeView) =>
  serve.serve === "Unreadable" ? serve : null;

/// And what a press came back with, where it was one of the two answers that is
/// not the machine read again. `undefined` before anything has been pressed.
const ungranted = (pressed: ServePress | undefined) =>
  pressed?.press === "Ungranted" ? pressed : null;
const troubled = (pressed: ServePress | undefined) =>
  pressed?.press === "Trouble" ? pressed : null;

/// What this machine's Tailscale is doing, read for the two panes that draw it.
///
/// A read of its own rather than a field of the settings, because none of it is
/// a setting: it is what the machine is doing, and the answer changes without
/// anybody having been to this page.
function useRemote() {
  return useReading(() => ({
    queryKey: ["remote"],
    queryFn: loadRemote,
    freshness: { reconcile: "tailscale" },
  }));
}

/// How things stand, in the one line somebody scanning the page is after.
///
/// The same sentence on the card and at the head of the pane, because it is the
/// same question in both: can this workbench be reached from a phone, and where
/// it cannot, what is in the way.
function standing(told: RemoteView): JSX.Element {
  return (
    <Choose>
      <Match when={absent(told)}>
        No Tailscale on this machine, so the workbench is reachable only from
        the machine it runs on.
      </Match>
      <Match when={down(told)}>
        Tailscale is installed here and not up, so there is no tailnet address
        to reach the workbench at.
      </Match>
      <Match when={unreadable(told)}>
        Tailscale answered in a way this build could not read, so what it is
        doing cannot be told from here.
      </Match>
      <Match when={up(told)} keyed>
        {(here) => (
          <Choose>
            <Match when={serving(here.serve)} keyed>
              {(on) => (
                <>
                  Served to the tailnet at{" "}
                  <span class={styles.address}>{on.address}</span>.
                </>
              )}
            </Match>
            <Match when={here.serve.serve === "Off"}>
              On the tailnet as <span class={styles.address}>{here.node}</span>,
              with the workbench not served to it.
            </Match>
            <Match when={here.serve.serve === "Unreadable"}>
              On the tailnet as <span class={styles.address}>{here.node}</span>.
              What is served to it could not be read.
            </Match>
          </Choose>
        )}
      </Match>
    </Choose>
  );
}

/// How this machine is reached, as the card that opens the section.
export function RemoteCard(props: {
  /// Whether the pane beside this is the one that is open.
  open: boolean;
  /// What pressing it does, which is opening that pane.
  press: () => void;
}): JSX.Element {
  const remote = useRemote();

  return (
    <Choose>
      <Match when={remote.isPending}>
        <Empty>Loading…</Empty>
      </Match>
      <Match when={remote.isError}>
        <ErrorLine>
          Could not read this machine's Tailscale: {remote.error?.message}
        </ErrorLine>
      </Match>
      <Match when={remote.data}>
        {(told) => (
          <CardButton
            as="article"
            class={styles.remoteCard}
            open={props.open}
            press={props.press}
          >
            <h2>Remote access</h2>

            <p class={styles.standing}>{standing(told())}</p>
          </CardButton>
        )}
      </Match>
    </Choose>
  );
}

/// And the whole of what the machine said, which is the details pane the card
/// opens.
export function RemotePane(props: {
  /// The way back to the settings, which is the pane this one was entered from.
  back: () => void;
}): JSX.Element {
  const queries = useQueryClient();
  const remote = useRemote();

  /// The serve switch, pressed.
  ///
  /// What a press that went through answers with is the machine read again, so
  /// it is written straight over the read the pane is drawn from: a second
  /// request would learn nothing the first one did not already say, and could
  /// only disagree with what is on screen while it was in flight.
  ///
  /// The other two answers are left in `press.data` and drawn from there. They
  /// are not errors — a refusal for want of the operator grant is a sentence
  /// with a command in it — so nothing here throws, and the next press replaces
  /// whichever of them is showing.
  const press = useMutation(() => ({
    mutationFn: (on: boolean) => pressServe({ on }),
    onSuccess: (pressed: ServePress) => {
      if (pressed.press === "Done") {
        queries.setQueryData(["remote"], pressed.reading);
      }
    },
  }));

  return (
    <>
      <PaneSticky>
        <PaneHead
          back={{ to: "Settings", go: props.back }}
          title="Remote access"
        />
      </PaneSticky>

      <Choose>
        <Match when={remote.isPending}>
          <Empty>Loading…</Empty>
        </Match>
        <Match when={remote.isError}>
          <ErrorLine>
            Could not read this machine's Tailscale: {remote.error?.message}
          </ErrorLine>
        </Match>
        <Match when={remote.data}>
          {(told) => (
            <div class={styles.remote}>
              <p class={styles.standing}>{standing(told())}</p>

              <Choose>
                {/* The one state whose answer is somewhere else entirely: a
                    pointer at where to get one, because nothing on this page
                    can install it. */}
                <Match when={absent(told())}>
                  <Note>
                    Verkstead reaches a phone over a tailnet: Tailscale puts
                    this machine on one and gives it a name of its own, and the
                    workbench is served on that name.{" "}
                    <a
                      class={styles.pointer}
                      href={DOWNLOAD}
                      target="_blank"
                      rel="noreferrer"
                    >
                      Install Tailscale
                    </a>{" "}
                    on this machine and join it to a tailnet, and this section
                    will say so.
                  </Note>
                </Match>

                {/* Installed and not up. What it printed is the useful half —
                    the line names the service to start — so it stands as it
                    came rather than reworded. */}
                <Match when={down(told())} keyed>
                  {(stopped) => (
                    <>
                      <Note>
                        Run <code>tailscale up</code> on this machine to join it
                        to a tailnet. This is what it said when it was asked:
                      </Note>
                      <p class={styles.trouble}>{stopped.trouble}</p>
                    </>
                  )}
                </Match>

                <Match when={unreadable(told())} keyed>
                  {(strange) => (
                    <>
                      <Note>
                        Tailscale is whatever this machine has, and this build
                        does not know the shape it answered in. This is what it
                        said:
                      </Note>
                      <p class={styles.trouble}>{strange.trouble}</p>
                    </>
                  )}
                </Match>

                <Match when={up(told())} keyed>
                  {(here) => (
                    <>
                      {/* Only where the serve state was readable. A switch over
                          an answer this build could not read would be offering
                          to turn on something that may already be on. */}
                      <Show when={!unread(here.serve)}>
                        <Switch
                          label="Serve the workbench to the tailnet"
                          on={Boolean(serving(here.serve))}
                          disabled={press.isPending}
                          flip={(on) => press.mutate(on)}
                        />

                        <Note>
                          Tailscale puts this machine's tailnet name in front of
                          the workbench over HTTPS, which is what a phone on the
                          tailnet opens. Where it stands is read off the machine,
                          so a serve set up in a terminal reads as on here and
                          this turns that one off.
                        </Note>
                      </Show>

                      {refused(press.data)}

                      <Show when={press.isError}>
                        <ErrorLine class={styles.failure}>
                          The serve could not be changed:{" "}
                          {press.error?.message}
                        </ErrorLine>
                      </Show>

                      <dl class={styles.readings}>
                        <dt>This machine on the tailnet</dt>
                        <dd class={styles.address}>{here.node}</dd>

                        {served(here.serve)}
                      </dl>

                      {/* And the way in, where there is an address to be let in
                          at. A machine serving nothing has none — and so has
                          nothing for a camera to be pointed at, and nothing for
                          a Reset to take back. */}
                      <Show when={here.link} keyed>
                        {(link) => <Reach link={link} />}
                      </Show>
                    </>
                  )}
                </Match>
              </Choose>
            </div>
          )}
        </Match>
      </Choose>
    </>
  );
}

/// How a phone gets in: the login link as a code to point a camera at, as text
/// to paste, and the press that takes it back.
///
/// Three things about one string, which is why they are one section rather than
/// three. The QR is for the phone in somebody's hand; the copy is for every
/// other way a link travels — a laptop on the same tailnet, a note to oneself —
/// and Reset key is what makes both of them recoverable, because a link handed
/// out is a link that cannot be taken back any other way.
///
/// The press answers with the machine read again, exactly as the serve switch's
/// does, so it is written straight over the read this pane is drawn from: the
/// code above redraws on the new key out of the answer rather than out of a
/// second request.
function Reach(props: { link: string }): JSX.Element {
  const queries = useQueryClient();

  const reset = useMutation(() => ({
    mutationFn: resetKey,
    onSuccess: (reading: RemoteView) =>
      queries.setQueryData(["remote"], reading),
  }));

  return (
    <section class={styles.reach}>
      <h3>Open the workbench on a phone</h3>

      <div class={styles.letIn}>
        <div class={styles.code}>
          <Qr of={props.link} label="The login link for this workbench" />
        </div>

        <div class={styles.beside}>
          <p class={styles.link}>{props.link}</p>
          <Copy of={props.link} class={styles.copy} />
        </div>
      </div>

      <Note>
        Scanning this opens the workbench on the phone and leaves it logged in.
        The link carries the workbench key, so anything holding it is in: it is
        worth as much as the workbench itself.
      </Note>

      <button
        type="button"
        class={styles.reset}
        disabled={reset.isPending}
        onClick={() => reset.mutate()}
      >
        {reset.isPending ? "Resetting…" : "Reset key"}
      </button>

      <Note>
        A new key over the old one, for a phone that was lost or a link that
        went where it should not have. Every other device is logged out by it,
        and the code above becomes the way back in.
      </Note>

      <Show when={reset.isError}>
        <ErrorLine class={styles.failure}>
          The key could not be reset: {reset.error?.message}
        </ErrorLine>
      </Show>
    </section>
  );
}

/// What is served to the tailnet, as the rows under the node's own name.
///
/// Three answers rather than two: a serve configuration that could not be read
/// is not a serve that is off, and saying *off* about one would be saying the
/// workbench is waiting to be served when nobody here knows whether it already
/// is.
function served(serve: ServeView): JSX.Element {
  return (
    <Choose>
      <Match when={serving(serve)} keyed>
        {(on) => (
          <>
            <dt>The workbench is served at</dt>
            <dd class={styles.address}>{on.address}</dd>
          </>
        )}
      </Match>
      <Match when={serve.serve === "Off"}>
        <dt>The workbench</dt>
        <dd>Not served to the tailnet.</dd>
      </Match>
      <Match when={unread(serve)} keyed>
        {(strange) => (
          <>
            <dt>The workbench</dt>
            <dd>
              What is served here could not be read.
              <span class={styles.trouble}>{strange.trouble}</span>
            </dd>
          </>
        )}
      </Match>
    </Choose>
  );
}

/// What a press said, where what it said was not the machine read again.
///
/// Two answers and neither of them an error the page can retry for the human.
/// The operator grant is a line to run in a terminal, and the next press is the
/// re-try — so it is drawn as the command it is, with what Tailscale said under
/// it. Anything else is the machine's own words and nothing to add to them.
///
/// Nothing at all before the first press, and nothing after one that went
/// through: the reading it answered with is already on the page.
function refused(pressed: ServePress | undefined): JSX.Element {
  return (
    <Choose>
      <Match when={ungranted(pressed)} keyed>
        {(denied) => (
          <>
            <Note>
              Tailscale will not set up a serve for the user this server runs
              as. Run this on the machine, then press the switch again:
            </Note>
            <p class={styles.command}>{denied.grant}</p>
            <p class={styles.trouble}>{denied.trouble}</p>
          </>
        )}
      </Match>
      <Match when={troubled(pressed)} keyed>
        {(bad) => (
          <>
            <Note>
              The serve could not be changed. This is what Tailscale said:
            </Note>
            <p class={styles.trouble}>{bad.trouble}</p>
          </>
        )}
      </Match>
    </Choose>
  );
}
