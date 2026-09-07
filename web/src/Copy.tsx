//! The copy button that stands beside a link.
//!
//! It says it copied and then stops saying it, because a clipboard write is
//! silent — nothing on the screen changes, and a press that looks like it did
//! nothing gets pressed again. A word for two seconds is the whole of the
//! feedback; there is nothing to undo and nothing to report.
//!
//! A clipboard the browser refuses — an insecure origin, a permission denied —
//! leaves the link itself, which is selectable text beside the button. So a
//! failure says nothing rather than opening a notice about a convenience.
//!
//! One component rather than one per pane: the Share pane's gist link and the
//! Remote access pane's login link are the same press about the same kind of
//! thing, and a second copy of it would be one to drift.

import { createSignal, type JSX } from "solid-js";

/// How long the word stands for, in milliseconds. Long enough to be read by
/// somebody who was looking at the button they pressed, and short enough that
/// it is gone before the next press.
const SAID_FOR = 2000;

/// A string, and the press that puts it on the clipboard.
export function Copy(props: {
  /// What is copied.
  of: string;
  /// And whatever the pane around it draws the button as.
  class?: string;
}): JSX.Element {
  const [copied, setCopied] = createSignal(false);

  const copy = () => {
    void navigator.clipboard
      ?.writeText(props.of)
      .then(() => {
        setCopied(true);
        setTimeout(() => setCopied(false), SAID_FOR);
      })
      .catch(() => {});
  };

  return (
    <button type="button" class={props.class} onClick={copy}>
      {copied() ? "Copied" : "Copy"}
    </button>
  );
}
