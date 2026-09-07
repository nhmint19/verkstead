//! What the Remote access section says of a machine, which is four different
//! things about four different machines.
//!
//! The point of the section is that its three unhappy states are told apart. A
//! machine with no Tailscale wants an install and nothing on the page can do
//! it; one whose daemon is not answering wants a command run in a terminal, and
//! the line it printed is what names the service; one that is up wants neither,
//! and the question about it is whether the workbench is served. A section that
//! collapsed any two of those into "not reachable" would leave the human
//! guessing which.
//!
//! Two halves mounted apart, because that is what they are: a card in the
//! middle pane saying how things stand, and the whole of what the machine said
//! in the details pane it opens.
//!
//! The reads are fixtures the server's own tests wrote, so what the page is
//! drawn from is the shape the endpoint really answers with — see
//! `crates/server/tests/remote.rs`, whose subject those shapes are.

import { fireEvent, render, screen, waitFor } from "@solidjs/testing-library";
import { QueryClient, QueryClientProvider } from "@tanstack/solid-query";
import type { JSX } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";

import type { RemoteView, ServePress } from "../src/api/types";
import { RemoteCard, RemotePane } from "../src/settings/Remote";
import absent from "./fixtures/remote-absent.json" with { type: "json" };
import down from "./fixtures/remote-down.json" with { type: "json" };
import off from "./fixtures/remote-off.json" with { type: "json" };
import unreadableServe from "./fixtures/remote-serve-unreadable.json" with { type: "json" };
import serving from "./fixtures/remote-serving.json" with { type: "json" };
import done from "./fixtures/serve-done.json" with { type: "json" };
import ungranted from "./fixtures/serve-ungranted.json" with { type: "json" };
import { json, whenever, serving as stubbing } from "./serving";

/// The five machines: no Tailscale at all, one whose daemon is not answering,
/// one that is up and serving the workbench, one that is up and serving
/// nothing, and one that is up and whose serve state could not be read.
const ABSENT = absent as RemoteView;
const DOWN = down as RemoteView;
const SERVING = serving as RemoteView;
const OFF = off as RemoteView;
const UNREADABLE_SERVE = unreadableServe as RemoteView;

/// And the two answers a press comes back with that are not the machine read
/// again — the operator grant, and the reading a press that went through
/// carries.
const DONE = done as ServePress;
const UNGRANTED = ungranted as ServePress;

afterEach(() => {
  vi.unstubAllGlobals();
});

function mounting(what: () => JSX.Element) {
  const queries = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  });

  return render(() => (
    <QueryClientProvider client={queries}>{what()}</QueryClientProvider>
  ));
}

/// A server answering the one read this section makes.
function theMachine(told: RemoteView) {
  return stubbing(whenever("/api/ui/remote", json(told)));
}

function mountCard(told: RemoteView) {
  theMachine(told);
  return mounting(() => <RemoteCard open={false} press={vi.fn()} />);
}

function mountPane(told: RemoteView) {
  theMachine(told);
  return mounting(() => <RemotePane back={vi.fn()} />);
}

describe("the card", () => {
  /// A machine with nothing installed says so on the card, because that is the
  /// one somebody scanning the page has the most to do about.
  it("says a machine with no Tailscale is only reachable from itself", async () => {
    mountCard(ABSENT);

    await waitFor(() =>
      expect(screen.getByText(/No Tailscale on this machine/)).toBeTruthy(),
    );
  });

  /// And one whose daemon is not answering is its own answer rather than a
  /// serve that is off: nothing about it says whether this workbench would be
  /// served, and a switch is not what fixes it.
  it("tells a Tailscale that is not up apart from a serve that is off", async () => {
    mountCard(DOWN);

    await waitFor(() =>
      expect(screen.getByText(/installed here and not up/)).toBeTruthy(),
    );

    expect(screen.queryByText(/not served to it/)).toBeNull();
  });

  /// A machine that is up and not serving names itself, and says the workbench
  /// is not on it.
  it("names the node of a machine that is up and serving nothing", async () => {
    mountCard(OFF);

    await waitFor(() =>
      expect(screen.getByText("workbench.tailnet-name.ts.net")).toBeTruthy(),
    );

    expect(screen.getByText(/not served to it/)).toBeTruthy();
  });

  /// And one that is serving says where, which is the address a phone is
  /// pointed at.
  it("says where a served workbench answers", async () => {
    mountCard(SERVING);

    await waitFor(() =>
      expect(
        screen.getByText("https://workbench.tailnet-name.ts.net"),
      ).toBeTruthy(),
    );
  });
});

describe("the pane", () => {
  /// The install pointer, which is the one thing this section has to say that
  /// nothing on the page can do — so it is a link out of Verkstead.
  it("points a machine with no Tailscale at where to get one", async () => {
    mountPane(ABSENT);

    const pointer = await waitFor(() => screen.getByText("Install Tailscale"));

    expect(pointer.getAttribute("href")).toBe("https://tailscale.com/download");
  });

  /// What the machine printed, shown as it came: the line names the service to
  /// start, and a word of it reworded is a command that does not work.
  it("shows what a Tailscale that is not up said, in its own words", async () => {
    mountPane(DOWN);

    await waitFor(() =>
      expect(
        screen.getByText(/sudo systemctl start tailscaled/),
      ).toBeTruthy(),
    );

    // And what to do about it, which is the one command that is the human's.
    expect(screen.getByText("tailscale up")).toBeTruthy();
  });

  /// A machine that is up reads as its two readings: what it is called on the
  /// tailnet, and what is served on that name.
  it("reads a machine that is up as its node and its serve", async () => {
    mountPane(SERVING);

    await waitFor(() =>
      expect(screen.getByText("This machine on the tailnet")).toBeTruthy(),
    );

    expect(screen.getByText("workbench.tailnet-name.ts.net")).toBeTruthy();
    expect(screen.getByText("The workbench is served at")).toBeTruthy();

    // Twice: the line at the head of the pane that sums the machine up, and
    // the reading itself under it.
    expect(
      screen.getAllByText("https://workbench.tailnet-name.ts.net"),
    ).toHaveLength(2);
  });

  /// And one that is up with nothing served says so, without an address —
  /// there is none until something is served.
  it("gives a machine serving nothing no address", async () => {
    mountPane(OFF);

    await waitFor(() =>
      expect(screen.getByText("Not served to the tailnet.")).toBeTruthy(),
    );

    expect(screen.queryByText(/^https:\/\//)).toBeNull();
  });
});

describe("the serve switch", () => {
  /// A server answering the read and the presses both.
  ///
  /// The read is held for its path, because the pane makes it whenever it likes
  /// and a test about a press should not have to say when. The presses go in the
  /// order given, which is what a re-try is made of: a refusal, and then the same
  /// press again once the grant has been run.
  function theMachinePressed(told: RemoteView, ...answers: Array<ServePress>) {
    return stubbing(
      whenever("/api/ui/remote", json(told)),
      ...answers.map((answer) => json(answer)),
    );
  }

  function theSwitch(): HTMLInputElement {
    return screen.getByRole("switch") as HTMLInputElement;
  }

  /// What the page put on the wire for the press it made.
  function pressed(fetching: ReturnType<typeof stubbing>): unknown {
    const put = fetching.mock.calls.find(
      ([path, init]) =>
        String(path) === "/api/ui/remote/serve" && init?.method === "POST",
    );

    expect(put, "expected the page to have pressed").toBeTruthy();
    return JSON.parse(String(put![1]?.body));
  }

  /// The position is the reading rather than anything the page remembers, which
  /// is what makes a serve set up in a terminal one this switch turns off.
  it("stands where the machine reads, not where it was last pressed", async () => {
    mountPane(SERVING);

    await waitFor(() => expect(theSwitch().checked).toBe(true));
  });

  it("stands off on a machine serving nothing", async () => {
    mountPane(OFF);

    await waitFor(() => expect(theSwitch().checked).toBe(false));
  });

  /// And a serve state this build could not read gets no switch at all: *cannot
  /// tell* under a control offering to turn *off* on would be the one sentence
  /// this section must never say.
  it("is not drawn where the serve state could not be read", async () => {
    mountPane(UNREADABLE_SERVE);

    await waitFor(() =>
      expect(screen.getAllByText(/could not be read/).length).toBeGreaterThan(0),
    );

    expect(screen.queryByRole("switch")).toBeNull();
  });

  /// A press says which way it is going, and the address it comes back with is
  /// what the pane then reads — the answer *is* the read, so nothing asks
  /// again.
  it("presses the serve on and reads the address off the answer", async () => {
    const fetching = theMachinePressed(OFF, DONE);
    mounting(() => <RemotePane back={vi.fn()} />);

    await waitFor(() => expect(theSwitch().checked).toBe(false));
    fireEvent.click(theSwitch());

    await waitFor(() =>
      expect(
        screen.getAllByText("https://workbench.tailnet-name.ts.net"),
      ).toHaveLength(2),
    );

    expect(pressed(fetching)).toEqual({ on: true });
    expect(theSwitch().checked).toBe(true);
  });

  /// And off the other way, which is the same press with the other value.
  it("presses the serve off", async () => {
    const fetching = theMachinePressed(SERVING, {
      press: "Done",
      reading: OFF,
    });
    mounting(() => <RemotePane back={vi.fn()} />);

    await waitFor(() => expect(theSwitch().checked).toBe(true));
    fireEvent.click(theSwitch());

    await waitFor(() =>
      expect(screen.getByText("Not served to the tailnet.")).toBeTruthy(),
    );

    expect(pressed(fetching)).toEqual({ on: false });
    expect(theSwitch().checked).toBe(false);
  });

  /// A serve Tailscale would not take draws the line that makes it take one —
  /// exactly as it is to be typed, because a word of it reworded is a command
  /// that does not work.
  it("draws the operator grant when a serve is refused", async () => {
    theMachinePressed(OFF, UNGRANTED);
    mounting(() => <RemotePane back={vi.fn()} />);

    await waitFor(() => expect(theSwitch().checked).toBe(false));
    fireEvent.click(theSwitch());

    await waitFor(() =>
      expect(
        screen.getByText("sudo tailscale set --operator=ada"),
      ).toBeTruthy(),
    );

    // With what Tailscale said under it, because the grant is this build's
    // reading of a refusal and the refusal is the machine's own.
    expect(screen.getByText("Access denied: serve config denied")).toBeTruthy();

    // And nothing moved: the switch is still where the machine reads.
    expect(theSwitch().checked).toBe(false);
  });

  /// The press after the grant has been run is the same press again, and it
  /// serves — which is the whole of what a re-try is here.
  it("serves on the press after the grant", async () => {
    theMachinePressed(OFF, UNGRANTED, DONE);
    mounting(() => <RemotePane back={vi.fn()} />);

    await waitFor(() => expect(theSwitch().checked).toBe(false));
    fireEvent.click(theSwitch());

    await waitFor(() =>
      expect(
        screen.getByText("sudo tailscale set --operator=ada"),
      ).toBeTruthy(),
    );

    fireEvent.click(theSwitch());

    await waitFor(() => expect(theSwitch().checked).toBe(true));

    // And the grant goes with the refusal it belonged to: there is nothing left
    // for anybody to run.
    expect(
      screen.queryByText("sudo tailscale set --operator=ada"),
    ).toBeNull();
  });
});
