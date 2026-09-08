//! The open pull requests Verkstead did not open, across every registered Repo.
//!
//! The door into the pipeline for work that is already somewhere else. A pull
//! request opened by hand, by a contributor or by the old tools is a branch with
//! a review on it and nothing driving the wrap-up, and this is the list the
//! *Wrap up a pull request* level under the compose box is drawn from.
//!
//! **Read, never kept.** GitHub owns this list — a pull request merges, another
//! is opened, a title is edited — so it is asked for every time it is drawn, the
//! way [`crate::stages::abandoned`] asks the repositories about their roadmaps.
//! A copy here would be a second opinion, wrong from the moment somebody pressed
//! *Merge*.
//!
//! **One `gh pr list` per Repo, in parallel.** Each is a network call, and the
//! human is waiting on the slowest of them rather than on the sum: a workbench
//! with six repositories registered should open in the time one GitHub takes to
//! answer.
//!
//! **And a Repo Verkstead cannot ask about says nothing at all.** No GitHub
//! remote, no `gh` on the PATH, nobody logged in, a GitHub that timed out — each
//! of those is a repository this list has no news about, and none of them is a
//! reason to fail the whole reading or to claim there is nothing open. What
//! Verkstead does not know is not an empty list, but it is not a broken page
//! either.

use std::collections::HashMap;

use verkstead_render::{OpenPullRequest, OpenPullRequestRepo};

use crate::github::{Gh, Listed};
use crate::store;

/// Every open pull request in the registered Repos, grouped by the Repo it was
/// read in, with the Conversation already holding each of them.
///
/// `held` is [`store::held_pull_requests`], read once for the whole list: which
/// pull requests are in the pipeline is a question about Verkstead's own record
/// rather than about GitHub, and asking it per row would be one query per pull
/// request in every repository the human has registered.
///
/// A Repo with nothing open contributes no group, exactly as a Repo with nothing
/// to continue contributes no notice — and so does one whose `gh` would not
/// answer, which is the same thing to a reader: there is nothing here to press.
pub(crate) async fn open(
    gh: &Gh,
    repos: Vec<store::Repo>,
    held: &HashMap<(i64, i64), i64>,
) -> Vec<OpenPullRequestRepo> {
    // Started together and collected in the order `repos` came in — by name,
    // which is the order every list of Repos is drawn in — so the level reads
    // the same however fast each GitHub answered.
    let asking: Vec<_> = repos
        .into_iter()
        .map(|repo| {
            let gh = gh.clone();

            (
                repo.id,
                repo.name.clone(),
                tokio::task::spawn_blocking(move || {
                    crate::github::open_pull_requests(&gh, &repo.path)
                }),
            )
        })
        .collect();

    let mut groups = Vec::new();

    for (repo_id, repo, asked) in asking {
        let listed = match asked.await {
            Ok(Ok(listed)) => listed,

            // GitHub could not be asked, or would not say. Logged and skipped:
            // the human asked what there is to wrap up, and a repository that
            // could not answer has told them nothing about that.
            Ok(Err(trouble)) => {
                tracing::debug!(
                    repo_id,
                    why = trouble.why(),
                    "a Repo's open pull requests could not be read through the host gh",
                );
                continue;
            }
            Err(error) => {
                tracing::error!(error = ?error, repo_id, "asking gh for a Repo's open pull requests failed");
                continue;
            }
        };

        let pull_requests: Vec<OpenPullRequest> = listed
            .into_iter()
            // A fork's head branch is in another repository, so nothing a
            // wrap-up did could be pushed to it — see [`Listed::fork`].
            .filter(|one| !one.fork)
            .map(|one| drawn(one, repo_id, held))
            .collect();

        if !pull_requests.is_empty() {
            groups.push(OpenPullRequestRepo {
                repo_id,
                repo,
                pull_requests,
            });
        }
    }

    groups
}

/// One listed pull request as a row draws it: what GitHub said, plus which
/// Conversation is already holding it.
fn drawn(one: Listed, repo_id: i64, held: &HashMap<(i64, i64), i64>) -> OpenPullRequest {
    OpenPullRequest {
        // By the Repo and the number together, because that pair is what a pull
        // request *is* to Verkstead: `#41` names something else in the next
        // repository along, or nothing at all.
        conversation_id: held.get(&(repo_id, one.number)).copied(),
        number: one.number,
        title: one.title,
        url: one.url,
        head: one.head,
        base: one.base,
        author: one.author,
    }
}
