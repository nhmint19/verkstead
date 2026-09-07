//! What the Remote access pane reads: whether this machine has a Tailscale,
//! whether it is up, what it is called on the tailnet and whether the tailnet
//! name is already in front of the workbench.
//!
//! Every one of those is a fact about the machine the tests happen to be running
//! on, which is the one machine this suite must not be asking about: a pane with
//! four things to say needs four machines to say them about. So `tailscale` is a
//! shell script here, the way `gh` is in the settings suite — the server runs
//! the program it is given and reads what it printed, and what it is given is a
//! script that prints what one of those four machines would.
//!
//! Unix only, for that reason and no other: the four cases are shapes of stdout
//! rather than anything about a platform, and a Windows run would be a second
//! machine reading the same JSON.
#![cfg(unix)]

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use verkstead_render::RemoteView;
use verkstead_server::remote::Tailscale;
use verkstead_server::{open_database, router_reading_tailscale};

/// The port the workbench is served on in this suite, and so the port a serve
/// has to be proxying to for the pane to call it this workbench's.
const PORT: u16 = 8422;

/// A machine on a tailnet, serving the workbench: `tailscale status` says the
/// daemon is running and names this node, and `tailscale serve status` says
/// HTTPS on that name is proxied to the workbench's port.
const SERVING: &str = r#"
case "$1" in
  status) printf '%s' '{"BackendState":"Running","Self":{"DNSName":"workbench.tailnet-name.ts.net."}}' ;;
  serve) printf '%s' '{"TCP":{"443":{"HTTPS":true}},"Web":{"workbench.tailnet-name.ts.net:443":{"Handlers":{"/":{"Proxy":"http://127.0.0.1:8422"}}}}}' ;;
esac
"#;

/// The same machine with no serve on it, which is what `tailscale serve status`
/// prints where nobody has ever run one.
const NOT_SERVING: &str = r#"
case "$1" in
  status) printf '%s' '{"BackendState":"Running","Self":{"DNSName":"workbench.tailnet-name.ts.net."}}' ;;
  serve) printf '%s' 'null' ;;
esac
"#;

/// A machine whose daemon is not running, which is what both commands do about
/// it: a non-zero exit, and the line naming the service to start.
const NO_DAEMON: &str = r#"
echo "failed to connect to local tailscaled; it doesn't appear to be running (sudo systemctl start tailscaled ?)" >&2
exit 1
"#;

/// And one whose daemon is running and which has joined no tailnet.
const NOT_LOGGED_IN: &str = r#"printf '%s' '{"BackendState":"NeedsLogin","Self":{}}'"#;

/// And one whose `tailscale` answers something this build has never seen.
const UNRECOGNISED: &str = r#"printf '%s' 'this is not the JSON you are looking for'"#;

/// A server reading a machine whose `tailscale` is `script`.
async fn app_reading(script: &str) -> (tempfile::TempDir, Router) {
    let dir = tempfile::tempdir().unwrap();
    let pool = open_database(&dir.path().join("verkstead.db"))
        .await
        .unwrap();

    let tailscale = Tailscale::running(
        vec![
            "/bin/sh".to_owned(),
            "-c".to_owned(),
            script.to_owned(),
            // `sh -c` gives `$0` the script's own name, so what Verkstead passes
            // lands in `$1` onwards.
            "tailscale".to_owned(),
        ],
        PORT,
    );

    (dir, router_reading_tailscale(pool, tailscale))
}

/// And a server on a machine with no `tailscale` at all, which is a program
/// that is not there rather than one that answers badly.
async fn app_without_tailscale() -> (tempfile::TempDir, Router) {
    let dir = tempfile::tempdir().unwrap();
    let pool = open_database(&dir.path().join("verkstead.db"))
        .await
        .unwrap();

    let tailscale = Tailscale::running(vec!["verkstead-no-such-tailscale".to_owned()], PORT);

    (dir, router_reading_tailscale(pool, tailscale))
}

async fn remote(app: &Router) -> RemoteView {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/ui/remote")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// A machine with nothing to run reads as a machine with nothing installed —
/// which is the one state whose answer is somewhere else entirely, and so the
/// one the pane has an install pointer for.
#[tokio::test]
async fn a_machine_with_no_tailscale_says_so() {
    let (_dir, app) = app_without_tailscale().await;

    assert_eq!(remote(&app).await, RemoteView::Absent);
}

/// A machine that has `tailscale` and no daemon answering is its own third
/// thing rather than a serve that is off: nothing about it says whether this
/// machine would be served, and turning a switch on would not be the fix.
#[tokio::test]
async fn a_daemon_that_is_not_answering_is_not_a_serve_that_is_off() {
    let (_dir, app) = app_reading(NO_DAEMON).await;

    let RemoteView::Down { trouble } = remote(&app).await else {
        panic!("a machine whose daemon is down should read as down");
    };

    // In the machine's own words, because they are the useful half: the line
    // tailscale prints is the one naming the service to start.
    assert!(
        trouble.contains("tailscaled"),
        "the daemon's own complaint should come back: {trouble}"
    );
}

/// And so is one whose daemon is up and which has joined no tailnet: there is no
/// node to name and no address to serve on, which is the same thing to do about
/// it and a different sentence to say.
#[tokio::test]
async fn a_machine_that_has_joined_no_tailnet_reads_as_down_too() {
    let (_dir, app) = app_reading(NOT_LOGGED_IN).await;

    let RemoteView::Down { trouble } = remote(&app).await else {
        panic!("a machine that is not on a tailnet should read as down");
    };

    assert!(
        trouble.contains("NeedsLogin"),
        "what the daemon reports should come back: {trouble}"
    );
}

/// A machine that is up names itself, and says where the workbench answers on
/// the tailnet.
#[tokio::test]
async fn a_machine_that_is_up_and_serving_names_the_node_and_the_address() {
    let (_dir, app) = app_reading(SERVING).await;

    assert_eq!(
        remote(&app).await,
        RemoteView::Up {
            node: "workbench.tailnet-name.ts.net".to_owned(),
            serve: verkstead_render::ServeView::On {
                address: "https://workbench.tailnet-name.ts.net".to_owned(),
            },
        }
    );
}

/// And one that is up with no serve on it says so, which is the state the
/// switch beside it turns on.
#[tokio::test]
async fn a_machine_that_is_up_and_serving_nothing_reads_as_off() {
    let (_dir, app) = app_reading(NOT_SERVING).await;

    assert_eq!(
        remote(&app).await,
        RemoteView::Up {
            node: "workbench.tailnet-name.ts.net".to_owned(),
            serve: verkstead_render::ServeView::Off,
        }
    );
}

/// An answer this build cannot read says exactly that. Tailscale is whatever
/// the host has and the JSON it prints is documented as subject to change, so a
/// shape nobody here has seen must not be folded into the nearest state with
/// room for it.
#[tokio::test]
async fn an_answer_this_build_cannot_read_is_neither_up_nor_down() {
    let (_dir, app) = app_reading(UNRECOGNISED).await;

    assert!(matches!(remote(&app).await, RemoteView::Unreadable { .. }));
}
