//! The pull request a draft is holding, and what taking it up would do.
//!
//! The other way work gets into the pipeline: a pull request Verkstead did not
//! open — by hand, by a contributor, by the old tools — taken up at its wrap-up,
//! with the ordinary Wrapping loop running from there. Nothing about that loop
//! knows or cares who opened the pull request; what was missing was a door into
//! it.
//!
//! **The card is what both composers draw**, the compose page's and the draft's
//! own, over the same box. Which is why it is here rather than in either of
//! them: what a human reads about a pull request they are about to take up
//! should not depend on whether the Conversation exists yet.
//!
//! And it stands *over* the box rather than in place of it, which is the whole
//! difference from the roadmap card beside it (see [`Adoption`](./Adoption.tsx)).
//! An adopted stage's brief is the repository's own and arrives with the
//! adoption, so there is nothing to write; a pull request brings words of its
//! own — a title and a description — and it is the one thing taken up that the
//! human is likeliest to have something to add to. So the box stays a box, and
//! what is left in it is the Brief.

import { Show, type JSX } from "solid-js";

import styles from "./TakeUp.module.css";

/// The pull request being held, as either composer names it.
///
/// Read off what that composer is holding rather than off GitHub. On the compose
/// page that is the row that was pressed; on a draft's own it is what the create
/// wrote down — and neither is asked again, a re-read being a `gh` call to name
/// something already on the screen. What GitHub says *now* is the take-up's
/// question.
export function HeldPullRequest(props: {
  /// What the Repo is called, because a number alone names nothing: `#41` is a
  /// different pull request in every repository.
  repo: string;
  number: number;
  title: string;
  /// Where it is, for the way out to GitHub itself.
  url: string;
  /// The branch the work is on, which is the branch the take-up checks out.
  head: string;
  /// And the branch it goes into.
  base: string;

  /// Put it down, where there is anywhere to put it down to.
  ///
  /// The compose page's own: clearing gives the box back the text that was
  /// stowed when the pull request was loaded over it. A draft's page has no
  /// such control — the Conversation was created holding this, and the way out
  /// of one is to close it.
  clear?: () => void;
}): JSX.Element {
  return (
    <div class={styles.held}>
      <p class={styles.line}>
        <a class={styles.what} href={props.url} target="_blank" rel="noreferrer">
          {props.repo} #{props.number}
        </a>
        <span class={styles.title}>{props.title}</span>

        {/* A mark rather than a word, as the companion rows' own is: the line
            beside it is what says which pull request is being put down. The
            screen reader gets the sentence. */}
        <Show when={props.clear}>
          {(clear) => (
            <button
              type="button"
              class={styles.clear}
              aria-label={`Clear #${props.number}`}
              onClick={() => clear()()}
            >
              ×
            </button>
          )}
        </Show>
      </p>

      <p class={styles.branches}>
        <code>{props.head}</code> into <code>{props.base}</code>
      </p>
    </div>
  );
}
