//! Whether a fresh Verkstead can do anything yet, asked of the server over the
//! viewer's namespace: the mode, the machine it is standing on, and what is
//! missing from it.
//!
//! Every one of those is a fact about the machine the tests happen to be
//! running on, which is the one machine this suite must not be asking about: a
//! wizard that has to say something about a box with no sandbox and a box with
//! everything cannot be asked about either on the box the suite is on. So the
//! machine is stated — a `PATH` of the suite's own making, with a `bwrap` that
//! is a shell script in it — the way `tailscale` is a script in the Remote
//! access suite and `gh` is one in the settings suite. What is asserted is what
//! the server made of it.
//!
//! The other half of the objective is not on the machine at all: a Profile is a
//! row in the store and an author is a line in `config.yaml`. Both go in
//! **before** the router is stood up, which is what makes a start over them a
//! start that has them — see [`served`]. The verdict is reached once, at
//! startup, so a suite that wrote them afterwards would be asking about a
//! server that came up without them.
//!
//! **Which is the whole of the last test here.** Deleting the last Profile once
//! the server is up is a step that stops standing met under a mode that stays
//! off — see ADR-0016, where a live predicate was rejected for exactly the page
//! it would put over work somebody has.
//!
//! Unix only: what a `bwrap` did is a shell script here, and a script is what a
//! machine with a shell can be handed. Every arm that is not this platform's is
//! a unit test in the module itself, which is where a stated platform can be
//! asked about without a machine to run one on.
#![cfg(unix)]

use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use tower::ServiceExt;
use verkstead_render::{Dependency, DependencyState, DependencyView, Distro, OnboardingView};
use verkstead_server::onboarding::Machine;
use verkstead_server::platform::Platform;
use verkstead_server::store::{Account, ProfileFacts};
use verkstead_server::{open_database, router_onboarding, store};

/// A program that is there and does nothing, which is the whole of what a row
/// probed by a `PATH` walk asks of one.
const A_PROGRAM: &str = "#!/bin/sh\nexit 0\n";

/// A `bwrap` that makes the namespace it was asked for, which is the one row
/// answered by a run rather than a walk.
const MAKES_A_NAMESPACE: &str = A_PROGRAM;

/// And one that is installed and will not: the words a machine with
/// unprivileged user namespaces switched off refuses in, which is the line
/// worth putting under the row.
const REFUSED: &str = "#!/bin/sh\n\
                       echo 'bwrap: No permissions to creating new namespace, \
                       likely because the kernel does not allow non-privileged \
                       user namespaces' >&2\n\
                       exit 1\n";

/// And what that machine said, as the row carries it.
const REFUSAL: &str = "bwrap: No permissions to creating new namespace, likely because \
                       the kernel does not allow non-privileged user namespaces";

/// What this machine says it is, which is the tab the wizard opens on.
const OS_RELEASE: &str = "NAME=\"Ubuntu\"\nID=ubuntu\nID_LIKE=debian\n";

/// Everything a machine needs: a sandbox that works, `git`, and one harness.
/// No `gh`, which is the row that never gates.
const EVERYTHING: &[(&str, &str)] = &[
    ("bwrap", MAKES_A_NAMESPACE),
    ("git", A_PROGRAM),
    ("claude", A_PROGRAM),
];

/// A Data Directory with a database in it, and nothing said yet about what this
/// start is to find there.
async fn ready() -> (tempfile::TempDir, SqlitePool) {
    let dir = tempfile::tempdir().unwrap();
    let pool = open_database(&dir.path().join("verkstead.db"))
        .await
        .unwrap();

    (dir, pool)
}

/// And a server over it, probing a machine whose `PATH` is one directory of the
/// suite's own making, holding `programs` — each a name and the script it is.
///
/// Stood up last, deliberately: the verdict is reached at startup, so whatever
/// this start is meant to have is put in the Data Directory before this is
/// called and nothing is written into it afterwards but the one delete the last
/// test makes.
fn served(dir: &Path, pool: &SqlitePool, programs: &[(&str, &str)]) -> Router {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).unwrap();

    for (name, script) in programs {
        let path = bin.join(name);
        std::fs::write(&path, script).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    let machine = Machine::stated(
        Platform::Linux,
        OsString::from(bin.as_os_str()),
        None,
        Some(OS_RELEASE.to_owned()),
    );

    router_onboarding(pool.clone(), dir.to_owned(), machine)
}

/// An Agent Profile in the store, which is the whole of what the accounts step
/// asks for.
async fn a_profile(pool: &SqlitePool, dir: &Path) -> i64 {
    let account = dir.join("account");
    std::fs::create_dir_all(&account).unwrap();

    let profile = store::create_profile(
        pool,
        &ProfileFacts {
            name: Some("Mine".to_owned()),
            account: Account::Claude {
                claude_dir: account.clone(),
                config_file: account.join("claude.json"),
            },
            models: vec!["sonnet".to_owned()],
        },
    )
    .await
    .unwrap()
    .expect("nothing else is called that");

    profile.id
}

/// And a git author in `config.yaml`, which is the whole of what the git step
/// asks for. Written rather than saved through the endpoint: what is being
/// stood up is a server that already has one.
fn an_author(dir: &Path) {
    std::fs::write(
        dir.join("config.yaml"),
        "git_author:\n  name: Tobias Cohen\n  email: tobi@tobico.net\n",
    )
    .unwrap();
}

async fn reading(app: &Router) -> OnboardingView {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/ui/onboarding")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();

    serde_json::from_slice(&body).expect("the reading the wizard is drawn from")
}

/// What one row of a reading says.
fn row(reading: &OnboardingView, dependency: Dependency) -> &DependencyView {
    reading
        .dependencies
        .iter()
        .find(|row| row.dependency == dependency)
        .expect("every dependency is a row")
}

/// A machine whose `bwrap` will not make a namespace comes up in onboarding
/// mode, with the sandbox row as the one thing holding it and the failed run's
/// own words under it.
///
/// Everything else about this start is met — `git`, a harness, a Profile and an
/// author — so what puts it in the mode is the sandbox and nothing else, which
/// is what makes the row the reading names the row that is wrong.
#[tokio::test]
async fn a_sandbox_that_will_not_run_is_a_start_in_onboarding_mode() {
    let (dir, pool) = ready().await;

    a_profile(&pool, dir.path()).await;
    an_author(dir.path());

    let app = served(
        dir.path(),
        &pool,
        &[
            ("bwrap", REFUSED),
            ("git", A_PROGRAM),
            ("claude", A_PROGRAM),
        ],
    );

    let reading = reading(&app).await;

    assert!(
        reading.mode,
        "a Verkstead that cannot make a sandbox can run no session, so the \
         wizard is the only page there is"
    );

    assert_eq!(
        row(&reading, Dependency::Sandbox).state,
        DependencyState::Absent {
            trouble: Some(REFUSAL.to_owned()),
        },
        "the machine's own line, which is the one that names what to change"
    );

    assert!(
        !reading.steps.dependencies,
        "the step the sandbox row is on is the unmet one"
    );
    assert!(
        reading.steps.accounts && reading.steps.git,
        "and the two beside it stand met: a Profile and an author are what the \
         store and the settings file already hold"
    );

    assert_eq!(
        reading.distro,
        Distro::Ubuntu,
        "and the tab that opens is the one this machine says it is"
    );
}

/// And a start with a sandbox, `git`, a harness, one Profile and an author
/// comes up with the mode off — which is the workbench opening as it always
/// did.
#[tokio::test]
async fn a_machine_with_the_objective_met_comes_up_with_the_mode_off() {
    let (dir, pool) = ready().await;

    a_profile(&pool, dir.path()).await;
    an_author(dir.path());

    let app = served(dir.path(), &pool, EVERYTHING);
    let reading = reading(&app).await;

    assert!(!reading.mode, "there is nothing to put in front of anybody");

    assert!(reading.steps.dependencies);
    assert!(reading.steps.accounts);
    assert!(reading.steps.git);

    assert_eq!(
        row(&reading, Dependency::Sandbox).state,
        DependencyState::Present,
        "the `bwrap` on this machine made the namespace it was asked for"
    );
    assert_eq!(
        row(&reading, Dependency::Claude).state,
        DependencyState::Present,
        "and one harness is what the objective asks for"
    );
    assert_eq!(
        row(&reading, Dependency::Codex).state,
        DependencyState::Absent { trouble: None },
        "the three beside it are absent, and hold nothing up"
    );
    assert_eq!(
        row(&reading, Dependency::Gh).state,
        DependencyState::Absent { trouble: None },
        "and `gh` is absent as well, GitHub being a choice rather than a dependency"
    );
}

/// Each of the three things the objective is holds the mode on by itself, and
/// the step that reads unmet is the one it belongs to.
///
/// Three starts rather than three requests, because the mode is settled once
/// per start: what is being asked is what a machine short of one thing comes up
/// as, and that is a fact about a start.
#[tokio::test]
async fn each_of_the_three_holds_the_mode_on_by_itself() {
    // No harness at all, and the other two given: a Verkstead with nothing to
    // run a session with.
    let without_a_harness = {
        let (dir, pool) = ready().await;

        a_profile(&pool, dir.path()).await;
        an_author(dir.path());

        let app = served(
            dir.path(),
            &pool,
            &[("bwrap", MAKES_A_NAMESPACE), ("git", A_PROGRAM)],
        );

        reading(&app).await
    };

    assert!(without_a_harness.mode);
    assert!(
        !without_a_harness.steps.dependencies,
        "a machine with no harness on it can launch nothing"
    );

    // Everything on the machine, and no Profile: nothing to launch a session
    // *under*.
    let without_a_profile = {
        let (dir, pool) = ready().await;

        an_author(dir.path());

        let app = served(dir.path(), &pool, EVERYTHING);

        reading(&app).await
    };

    assert!(without_a_profile.mode);
    assert!(
        !without_a_profile.steps.accounts,
        "an account on the machine is not an Agent Profile until it is saved as one"
    );

    // And everything but the author, which is what git asks for on every commit
    // a session makes.
    let without_an_author = {
        let (dir, pool) = ready().await;

        a_profile(&pool, dir.path()).await;

        let app = served(dir.path(), &pool, EVERYTHING);

        reading(&app).await
    };

    assert!(without_an_author.mode);
    assert!(
        !without_an_author.steps.git,
        "a session that committed as nobody is a session git would refuse"
    );
}

/// The verdict is fixed for the length of a run: deleting the last Profile
/// while the server is up leaves the mode where the start put it, and the step
/// beside it says what is now true.
///
/// The two are meant to disagree. A live predicate would put a first-run page
/// over work somebody has, which is what ADR-0016 rejected it for — so what a
/// Profile that has gone gets is the settings page's own empty state, and
/// nothing here.
#[tokio::test]
async fn deleting_the_last_profile_mid_run_leaves_the_mode_where_it_was() {
    let (dir, pool) = ready().await;

    let profile = a_profile(&pool, dir.path()).await;
    an_author(dir.path());

    let app = served(dir.path(), &pool, EVERYTHING);

    assert!(
        !reading(&app).await.mode,
        "this start found the objective met"
    );

    store::delete_profile(&pool, profile).await.unwrap();

    let reading = reading(&app).await;

    assert!(
        !reading.mode,
        "the mode is off for the rest of this run, whatever has been deleted since"
    );
    assert!(
        !reading.steps.accounts,
        "and the step says what is true now: there is no Profile left to launch under"
    );
}

/// Where the golden fixtures are written, relative to this crate — the same
/// directory `ui_content` and `nudges` write the other endpoints' payloads to.
const FIXTURES: &str = "../../web/tests/fixtures";

/// Leave the viewer's own tests a reading of each shape the wizard is drawn
/// over, exactly as this server writes one.
///
/// Committed, and rewritten by every run of this test: the diff is the review.
/// The wizard's component tests are fed from these rather than from a payload
/// somebody typed out, so a field added on this side that nobody carried across
/// shows up as a failing fixture rather than as a page drawing the wrong thing.
///
/// Three of them, because three states are what the frame has to draw: a start
/// with nothing at all, one part way through, and one whose objective is met —
/// which is the only one of the three the wizard is not the page for.
///
/// Nothing here is read off the machine the suite is on. The `PATH`, the
/// `bwrap` and `/etc/os-release` are all [`served`]'s own, so a run today and a
/// run on another box write the same bytes.
#[tokio::test]
async fn the_viewers_own_tests_are_fed_from_here() {
    // A machine with nothing on it and a Data Directory with nothing in it:
    // every row absent and all three steps unmet, which is a first start on a
    // box somebody has just installed Verkstead on.
    let (dir, pool) = ready().await;
    let app = served(dir.path(), &pool, &[]);
    write("onboarding-fresh.json", &reading(&app).await);

    // The dependencies settled and nothing else: the step that is met, the two
    // that are not, and the mode still on.
    let (dir, pool) = ready().await;
    let app = served(dir.path(), &pool, EVERYTHING);
    write("onboarding-part-way.json", &reading(&app).await);

    // And the objective met, which is the reading that says the wizard is no
    // page at all.
    let (dir, pool) = ready().await;
    a_profile(&pool, dir.path()).await;
    an_author(dir.path());
    let app = served(dir.path(), &pool, EVERYTHING);
    write("onboarding-ready.json", &reading(&app).await);
}

/// One fixture, as the server would have written it.
fn write(name: &str, reading: &OnboardingView) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURES);
    std::fs::create_dir_all(&dir).unwrap();

    let mut pretty = serde_json::to_string_pretty(reading).unwrap();
    pretty.push('\n');

    std::fs::write(dir.join(name), pretty).unwrap();
}
