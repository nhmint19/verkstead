//! Registering a Repo over the viewer's namespace: what gets on the list, what
//! is refused before it can, what one of them says when it is opened, and what
//! taking one off the registry does to the list it was on.
//!
//! Every refusal here is asked of the *server*, through the endpoint, rather
//! than of the checks underneath it: a browser that skipped the form, or a
//! `curl` that never saw one, meets the same answers.
//!
//! Where a repository *is* is not one of the refusals. Anywhere the server can
//! read is somewhere a Repo can be registered from, whatever the installation
//! was started with.
//!
//! And the one thing there is to say to a Repo that is already registered: how
//! it resolves a merge conflict, which is an override of the global setting and
//! so is nothing at all until somebody says something.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use tower::ServiceExt;
use verkstead_render::{ConflictResolution, Registered, RepoEntry, RepoRemoved, RepoView};
use verkstead_server::{open_database, router_keeping, store};

/// A router, plus the Data Directory holding its database alive.
async fn workbench() -> (tempfile::TempDir, Router) {
    let (dir, _pool, app) = workbench_and_pool().await;

    (dir, app)
}

/// The same, with the pool beside it — for the tests that put Conversations on
/// a Repo, which is the one thing they need that this namespace has no endpoint
/// for.
async fn workbench_and_pool() -> (tempfile::TempDir, SqlitePool, Router) {
    let dir = tempfile::tempdir().unwrap();
    let pool = open_database(&dir.path().join("verkstead.db"))
        .await
        .unwrap();

    let data_dir = dir.path().to_owned();

    (dir, pool.clone(), router_keeping(pool, data_dir))
}

/// A git repository at `path`, with one commit on `main` so it has a branch to
/// call its default.
fn repository(path: PathBuf) -> PathBuf {
    std::fs::create_dir_all(&path).unwrap();
    git(&path, &["init", "--initial-branch", "main"]);
    git(&path, &["config", "user.email", "test@verkstead.invalid"]);
    git(&path, &["config", "user.name", "Verkstead Test"]);
    std::fs::write(path.join("README.md"), "# a repository\n").unwrap();
    git(&path, &["add", "README.md"]);
    git(&path, &["commit", "-m", "first"]);

    path
}

fn git(dir: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("git should be on the PATH for these tests");

    assert!(status.success(), "git {args:?} failed in {}", dir.display());
}

/// Ask to register a path, and read back what the server made of it.
async fn register(app: &Router, path: &Path) -> Registered {
    post(app, "/api/ui/repos", &serde_json::json!({ "path": path })).await
}

/// The Repo a registration that landed hands back — and the assertion that it
/// landed, which is the same thing: an outcome that carries a Repo is one that
/// left a Repo registered.
#[track_caller]
fn added(outcome: Registered) -> RepoEntry {
    match outcome {
        Registered::Added(repo) => repo,
        refused => panic!("the registration was refused: {refused:?}"),
    }
}

/// And the Repo a path already registered hands back, which is the same Repo the
/// first registration made.
#[track_caller]
fn already(outcome: Registered) -> RepoEntry {
    match outcome {
        Registered::AlreadyRegistered(repo) => repo,
        other => panic!("the path was not already registered: {other:?}"),
    }
}

/// The same, for a path that is not one the filesystem can hand back — a string
/// typed into the form.
async fn register_text(app: &Router, path: &str) -> Registered {
    post(app, "/api/ui/repos", &serde_json::json!({ "path": path })).await
}

async fn listed(app: &Router) -> Vec<RepoEntry> {
    get(app, "/api/ui/repos").await
}

/// The branches of one registered Repo, which is what the base dropdown offers.
async fn branches(app: &Router, id: i64) -> Vec<String> {
    get(app, &format!("/api/ui/repos/{id}/branches")).await
}

/// Ask for one to be taken off the registry, and read back what the server made
/// of that.
async fn remove(app: &Router, id: i64) -> RepoRemoved {
    post(
        app,
        &format!("/api/ui/repos/{id}/remove"),
        &serde_json::Value::Null,
    )
    .await
}

/// Say how one Repo is to resolve a conflict from now on — or, with `None`, that
/// it is to go back to whatever every other Repo does.
async fn resolve(app: &Router, id: i64, resolution: Option<ConflictResolution>) -> RepoView {
    post(
        app,
        &format!("/api/ui/repos/{id}/resolution"),
        &serde_json::json!({ "resolution": resolution }),
    )
    .await
}

async fn get<T: DeserializeOwned>(app: &Router, path: &str) -> T {
    let (status, body) = fetch(
        app,
        Request::builder().uri(path).body(Body::empty()).unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "GET {path} failed: {body}");
    read(&body)
}

async fn post<T: DeserializeOwned>(app: &Router, path: &str, body: &serde_json::Value) -> T {
    let (status, body) = fetch(
        app,
        Request::builder()
            .method("POST")
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(serde_json::to_vec(body).unwrap()))
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "POST {path} failed: {body}");
    read(&body)
}

async fn fetch(app: &Router, request: Request<Body>) -> (StatusCode, String) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

fn read<T: DeserializeOwned>(body: &str) -> T {
    serde_json::from_str(body).unwrap_or_else(|err| panic!("reading {body:?}: {err}"))
}

#[tokio::test]
async fn a_repository_registers_and_appears_on_the_list() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    let registered = added(register(&app, &repo).await);

    let repos = listed(&app).await;
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "verkstead");
    assert_eq!(
        repos[0].path,
        repo.canonicalize().unwrap().to_str().unwrap()
    );
    assert_eq!(repos[0].default_branch, "main");

    // And the registration hands back that same row rather than an outcome the
    // caller has to go looking for the Repo behind — the path it recorded is
    // the resolved one, which is not the one that was typed.
    assert_eq!(registered, repos[0]);
}

#[tokio::test]
async fn nothing_is_registered_to_begin_with() {
    let (_dir, app) = workbench().await;

    assert!(listed(&app).await.is_empty());
}

/// The boundary is gone: a repository nowhere near anything the server was
/// started with — its own Data Directory included — registers like any other,
/// and what is stored is where it really is.
#[tokio::test]
async fn a_repository_outside_everything_the_server_was_started_with_registers() {
    let root = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;

    let elsewhere = repository(root.path().join("elsewhere"));

    added(register(&app, &elsewhere).await);

    let repos = listed(&app).await;
    assert_eq!(repos.len(), 1);
    assert_eq!(
        repos[0].path,
        elsewhere.canonicalize().unwrap().to_str().unwrap()
    );
}

/// What is registered is the resolved path rather than the one that was typed:
/// a symlink is followed first, so the row names the directory a session will
/// actually stand in.
///
/// Made where a link can be made without asking anybody's permission, which is
/// both Unixes and not Windows. The resolving is the same everywhere — it is
/// `canonicalize` — so what is lost there is the making of the link rather than
/// any of the reasoning.
#[cfg(unix)]
#[tokio::test]
async fn a_repository_reached_through_a_symlink_is_stored_where_it_really_is() {
    let root = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;

    let elsewhere = repository(root.path().join("elsewhere"));
    let link = root.path().join("looks-elsewhere");
    std::os::unix::fs::symlink(&elsewhere, &link).unwrap();

    added(register(&app, &link).await);

    assert_eq!(
        listed(&app).await[0].path,
        elsewhere.canonicalize().unwrap().to_str().unwrap()
    );
}

#[tokio::test]
async fn a_directory_that_is_not_a_git_repository_is_refused() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;

    let plain = src.path().join("notes");
    std::fs::create_dir(&plain).unwrap();

    assert_eq!(register(&app, &plain).await, Registered::NotARepository);
}

/// A directory *in* a repository is not the repository: everything Verkstead
/// later builds hangs off the root, so a subdirectory would put a Conversation's
/// worktree somewhere nobody meant.
#[tokio::test]
async fn a_subdirectory_of_a_repository_is_not_the_repository() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    let inside = repo.join("crates");
    std::fs::create_dir(&inside).unwrap();

    assert_eq!(register(&app, &inside).await, Registered::NotARepository);
}

#[tokio::test]
async fn a_path_with_nothing_at_it_is_refused_as_missing() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;

    assert_eq!(
        register(&app, &src.path().join("never-made")).await,
        Registered::Missing
    );
}

#[tokio::test]
async fn a_relative_path_is_refused_rather_than_resolved() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    repository(src.path().join("verkstead"));

    assert_eq!(
        register_text(&app, "verkstead").await,
        Registered::NotAbsolute
    );
}

#[tokio::test]
async fn a_repo_already_registered_is_refused_however_its_path_is_spelled() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    let first = added(register(&app, &repo).await);

    // The same directory, spelled its way out and back again.
    let roundabout = repo.join("..").join("verkstead");

    // And the same Repo comes back with the refusal, which is what makes it a
    // repository to land a draft on rather than a dead end: a dropdown that
    // registered a path somebody had registered already has the Repo it named.
    assert_eq!(already(register(&app, &roundabout).await), first);

    assert_eq!(listed(&app).await.len(), 1);
}

/// The standalone install: no unit, no flags, nothing configured anywhere. It
/// registers a repository like any other, which is what a bare binary being
/// usable out of the box means.
#[tokio::test]
async fn a_server_told_nothing_at_all_registers_all_the_same() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    added(register(&app, &repo).await);
    assert_eq!(listed(&app).await.len(), 1);
}

/// What a Conversation branches from: the remote's idea of the default branch
/// wins over whatever happens to be checked out, because that is what everyone
/// working on the repository means by it.
#[tokio::test]
async fn the_default_branch_is_what_the_remote_calls_it() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    // A remote of its own, pointing back at itself: enough for `origin/HEAD` to
    // exist and name a branch, without a network anywhere.
    git(&repo, &["remote", "add", "origin", repo.to_str().unwrap()]);
    git(&repo, &["fetch", "--quiet", "origin"]);
    git(
        &repo,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    git(&repo, &["checkout", "--quiet", "-b", "some-feature"]);

    added(register(&app, &repo).await);
    assert_eq!(listed(&app).await[0].default_branch, "main");
}

/// A repository with nothing checked out has no branch to work from, and
/// inventing one would put work on a branch nobody chose.
#[tokio::test]
async fn a_repository_with_no_branch_to_call_its_default_is_refused() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    git(&repo, &["checkout", "--quiet", "--detach", "HEAD"]);

    assert_eq!(register(&app, &repo).await, Registered::NoDefaultBranch);
}

/// The list a drafting Conversation picks what it comes off out of: every
/// branch the repository has, local and remote-tracking both.
///
/// `origin/HEAD` is left out of it. It is a symbolic ref — another name for a
/// branch that is already on the list — and offering it twice would be offering
/// a choice that is not one.
#[tokio::test]
async fn a_repos_branches_are_the_local_and_remote_tracking_ones() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    git(&repo, &["branch", "release"]);
    git(&repo, &["remote", "add", "origin", repo.to_str().unwrap()]);
    git(&repo, &["fetch", "--quiet", "origin"]);
    git(
        &repo,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );

    added(register(&app, &repo).await);
    let id = listed(&app).await[0].id;

    assert_eq!(
        branches(&app, id).await,
        vec![
            "main".to_owned(),
            "release".to_owned(),
            "origin/main".to_owned(),
            "origin/release".to_owned(),
        ],
        "the locals first, then what the remote is carrying",
    );
}

/// A Repo that is not registered has no branches to read, and saying so is a
/// refusal rather than an empty list: an empty list is a repository with
/// nothing on it, which is a different thing to be told.
#[tokio::test]
async fn the_branches_of_a_repo_that_is_not_there_are_refused() {
    let (_dir, app) = workbench().await;

    for asked in ["404", "not-a-number"] {
        let (status, _) = fetch(
            &app,
            Request::builder()
                .uri(format!("/api/ui/repos/{asked}/branches"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;

        assert_eq!(status, StatusCode::NOT_FOUND, "asking about {asked}");
    }
}

/// One Repo opened, which is what its card in the settings leads to: the row's
/// own three facts, plus everything the card had no room for.
///
/// The roadmaps are the same reading the notice under the new-conversation box
/// makes — `ui_content.rs` is where what that finds is pinned — so what is
/// asserted here is that a repository holding none says so with an empty list
/// rather than by leaving the field out.
#[tokio::test]
async fn a_repo_opened_carries_its_branches_its_work_and_its_roadmaps() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, pool, app) = workbench_and_pool().await;
    let repo = repository(src.path().join("verkstead"));
    git(&repo, &["branch", "release"]);

    added(register(&app, &repo).await);
    let id = listed(&app).await[0].id;

    // Three Conversations on it: one still going, and two that are over each
    // way there is to be over.
    for (branch, state) in [
        ("rate-limiting", None),
        ("pane-paths", Some(store::Lifecycle::Done)),
        ("dropped", Some(store::Lifecycle::Closed)),
    ] {
        let started = store::start_conversation(&pool, id, branch)
            .await
            .unwrap()
            .unwrap();

        if let Some(state) = state {
            store::set_state(&pool, started, state).await.unwrap();
        }
    }

    let opened: RepoView = get(&app, &format!("/api/ui/repos/{id}")).await;

    assert_eq!(opened.id, id);
    assert_eq!(opened.name, "verkstead");
    assert_eq!(opened.path, repo.canonicalize().unwrap().to_str().unwrap());
    assert_eq!(opened.default_branch, "main");
    assert_eq!(
        opened.branches,
        vec!["main".to_owned(), "release".to_owned()],
        "the same list the base dropdown is filled from",
    );
    assert_eq!(opened.live, 1);
    assert_eq!(opened.finished, 2, "Done and Closed counted together");
    assert!(
        opened.roadmaps.is_empty(),
        "a repository with no roadmaps has none waiting: {:?}",
        opened.roadmaps,
    );
}

/// A registered Repo says nothing about how it resolves a conflict until
/// somebody tells it, and then it says that until they take it back.
///
/// `null` is *whatever the settings page says for every Repo* rather than
/// *merge*: the two are the same answer today and stop being the same the moment
/// the global is changed, so what a Repo nobody has been to holds is nothing at
/// all.
#[tokio::test]
async fn a_repos_resolution_is_said_taken_back_and_read_off_the_pane() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    added(register(&app, &repo).await);
    let id = listed(&app).await[0].id;

    let opened: RepoView = get(&app, &format!("/api/ui/repos/{id}")).await;
    assert_eq!(
        opened.conflict_resolution, None,
        "a Repo nobody has told overrides nothing",
    );

    let saved = resolve(&app, id, Some(ConflictResolution::Rebase)).await;
    assert_eq!(
        saved.conflict_resolution,
        Some(ConflictResolution::Rebase),
        "the answer is the Repo as it now stands, which is what the pane draws",
    );

    let read: RepoView = get(&app, &format!("/api/ui/repos/{id}")).await;
    assert_eq!(read.conflict_resolution, Some(ConflictResolution::Rebase));

    assert_eq!(
        resolve(&app, id, Some(ConflictResolution::Merge))
            .await
            .conflict_resolution,
        Some(ConflictResolution::Merge),
        "and either word can be the override, a Repo pinned to a merge being a \
         real thing to say where the global is a rebase",
    );

    assert_eq!(
        resolve(&app, id, None).await.conflict_resolution,
        None,
        "and clearing it puts the Repo back to whatever every other one does",
    );
}

/// A Repo nothing is registered under has nothing to be told, and saying so is
/// the same refusal opening one gives: a pane somebody left open in another tab
/// while the Repo was taken away.
#[tokio::test]
async fn a_repo_that_is_not_there_cannot_be_told_how_to_resolve_a_conflict() {
    let (_dir, app) = workbench().await;

    for asked in ["404", "not-a-number"] {
        let (status, _) = fetch(
            &app,
            Request::builder()
                .method("POST")
                .uri(format!("/api/ui/repos/{asked}/resolution"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    serde_json::json!({ "resolution": "Merge" }).to_string(),
                ))
                .unwrap(),
        )
        .await;

        assert_eq!(status, StatusCode::NOT_FOUND, "telling {asked}");
    }
}

/// A Repo that is not registered has nothing to open, and saying so is a
/// refusal: the pane reads it as the repo being gone — a link followed after
/// somebody took it away — rather than as a Repo with nothing on it.
#[tokio::test]
async fn a_repo_that_is_not_there_cannot_be_opened() {
    let (_dir, app) = workbench().await;

    for asked in ["404", "not-a-number"] {
        let (status, _) = fetch(
            &app,
            Request::builder()
                .uri(format!("/api/ui/repos/{asked}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;

        assert_eq!(status, StatusCode::NOT_FOUND, "opening {asked}");
    }
}

/// And a Repo that was taken off the registry has nothing to open either. It is
/// still in the table — every Conversation ever worked in it names it — but
/// nothing is registered under that id any more, and the pane reads that as the
/// repo being gone rather than drawing one with a Remove button on it.
#[tokio::test]
async fn a_repo_that_was_removed_cannot_be_opened() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    added(register(&app, &repo).await);
    let id = listed(&app).await[0].id;

    assert_eq!(remove(&app, id).await, RepoRemoved::Removed);

    let (status, _) = fetch(
        &app,
        Request::builder()
            .uri(format!("/api/ui/repos/{id}"))
            .body(Body::empty())
            .unwrap(),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

/// A Repo taken off the registry is off every list that offers Repos for new
/// work — this one, the compose page's Repo dropdown behind it, and the
/// roadmaps there are to adopt, all of which are the same read.
#[tokio::test]
async fn a_removed_repo_is_off_the_list() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    added(register(&app, &repo).await);
    let id = listed(&app).await[0].id;

    assert_eq!(remove(&app, id).await, RepoRemoved::Removed);
    assert!(listed(&app).await.is_empty());

    // And the roadmap notice, which is a read of its own over the same list.
    let waiting: Vec<serde_json::Value> = get(&app, "/api/ui/abandoned-roadmaps").await;
    assert!(waiting.is_empty(), "an unregistered Repo offers nothing");
}

/// Work still going on in a repository is the reason to keep it registered, so
/// the removal is refused with the reason the pane says out loud — and the Repo
/// is where it was.
#[tokio::test]
async fn a_repo_with_live_work_on_it_is_refused() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, pool, app) = workbench_and_pool().await;
    let repo = repository(src.path().join("verkstead"));

    added(register(&app, &repo).await);
    let id = listed(&app).await[0].id;

    let going = store::start_conversation(&pool, id, "rate-limiting")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(remove(&app, id).await, RepoRemoved::InUse);
    assert_eq!(listed(&app).await.len(), 1, "nothing was taken away");

    // Closed is over, and what is over is no reason to hold the registration.
    store::set_state(&pool, going, store::Lifecycle::Closed)
        .await
        .unwrap();

    assert_eq!(remove(&app, id).await, RepoRemoved::Removed);
}

/// An id nothing is registered under is a named outcome rather than a status:
/// one already taken away, one that never was, and one that is not a number at
/// all are the same sentence.
#[tokio::test]
async fn there_is_nothing_to_remove_twice() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    added(register(&app, &repo).await);
    let id = listed(&app).await[0].id;

    assert_eq!(remove(&app, id).await, RepoRemoved::Removed);
    assert_eq!(remove(&app, id).await, RepoRemoved::NoSuchRepo);
    assert_eq!(remove(&app, 404).await, RepoRemoved::NoSuchRepo);

    let refused: RepoRemoved = post(
        &app,
        "/api/ui/repos/not-a-number/remove",
        &serde_json::Value::Null,
    )
    .await;
    assert_eq!(refused, RepoRemoved::NoSuchRepo);
}

/// And registering the same repository again brings it back rather than being
/// refused as registered already — which is what makes a removal something the
/// human can undo.
#[tokio::test]
async fn registering_a_removed_repo_again_brings_it_back() {
    let src = tempfile::tempdir().unwrap();
    let (_dir, app) = workbench().await;
    let repo = repository(src.path().join("verkstead"));

    added(register(&app, &repo).await);
    let id = listed(&app).await[0].id;
    assert_eq!(remove(&app, id).await, RepoRemoved::Removed);

    added(register(&app, &repo).await);

    let back = listed(&app).await;
    assert_eq!(back.len(), 1);
    assert_eq!(back[0].id, id, "the same Repo, under the id it always had");
}
