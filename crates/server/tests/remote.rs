//! What the Remote access pane reads: whether this machine has a Tailscale,
//! whether it is up, what it is called on the tailnet and whether the tailnet
//! name is already in front of the workbench — and the one thing on it that is
//! pressed, which is putting that name there and taking it off again.
//!
//! Every one of those is a fact about the machine the tests happen to be running
//! on, which is the one machine this suite must not be asking about: a pane with
//! four things to say needs four machines to say them about. So `tailscale` is a
//! shell script here, the way `gh` is in the settings suite — the server runs
//! the program it is given and reads what it printed, and what it is given is a
//! script that prints what one of those four machines would.
//!
//! The press needs a fifth, and a different kind of one: a machine that answers
//! differently after it has been pressed. That one keeps its serve in a file and
//! reads it back out — see [`switchable`] — because what makes the switch worth
//! pressing is precisely that the read afterwards says something else.
//!
//! Unix only, for that reason and no other: the cases are shapes of stdout
//! rather than anything about a platform, and a Windows run would be a second
//! machine reading the same JSON.
#![cfg(unix)]

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use verkstead_render::{RemoteView, ServePress};
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

/// A machine on a tailnet whose serve is whatever this suite last did to it.
///
/// The four cases above are machines that answer the same thing however often
/// they are asked, which is all a read needs. A press is the other half: what
/// makes it worth pressing is that the next read says something different, so
/// this `tailscale` keeps its serve in a file and both halves go through it.
///
/// `--bg` is refused until the operator file is there, which is what the grant
/// is here: `tailscale serve` from a process that is neither root nor the
/// tailnet's operator is denied by the daemon, and the file stands for the
/// `sudo tailscale set --operator=…` somebody runs in a terminal to lift it.
fn switchable(state: &std::path::Path) -> String {
    let state = state.display();

    format!(
        r#"
case "$1" in
  status) printf '%s' '{{"BackendState":"Running","Self":{{"DNSName":"workbench.tailnet-name.ts.net."}}}}' ;;
  serve)
    case "$2" in
      status)
        if [ -e "{state}/serving" ]; then
          printf '%s' '{{"TCP":{{"443":{{"HTTPS":true}}}},"Web":{{"workbench.tailnet-name.ts.net:443":{{"Handlers":{{"/":{{"Proxy":"http://127.0.0.1:8422"}}}}}}}}}}'
        else
          printf '%s' 'null'
        fi ;;
      --bg)
        if [ -e "{state}/operator" ]; then
          : > "{state}/serving"
        else
          echo "Access denied: serve config denied" >&2
          exit 1
        fi ;;
      --https=443) rm -f "{state}/serving" ;;
    esac ;;
esac
"#
    )
}

/// The user this suite's server runs as, so the grant it hands back is a line
/// this file can write down. Whoever is running `cargo test` is nobody's
/// business here — see [`Tailscale::as_user`].
const WHO: &str = "ada";

/// A server whose `tailscale` is the switchable machine above, with the state
/// directory that machine keeps its serve in.
async fn app_pressing() -> (tempfile::TempDir, Router) {
    let dir = tempfile::tempdir().unwrap();
    let pool = open_database(&dir.path().join("verkstead.db"))
        .await
        .unwrap();

    let tailscale = Tailscale::running(
        vec![
            "/bin/sh".to_owned(),
            "-c".to_owned(),
            switchable(dir.path()),
            // `sh -c` gives `$0` the script's own name, so what Verkstead passes
            // lands in `$1` onwards.
            "tailscale".to_owned(),
        ],
        PORT,
    )
    .as_user(WHO.to_owned());

    (dir, router_reading_tailscale(pool, tailscale))
}

/// Press the switch, and read what came of it.
async fn press(app: &Router, on: bool) -> ServePress {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/ui/remote/serve")
                .header("content-type", "application/json")
                .body(Body::from(format!(r#"{{"on":{on}}}"#)))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// The grant, run in a terminal — which here is the file the machine above
/// looks for.
fn granted(dir: &tempfile::TempDir) {
    std::fs::write(dir.path().join("operator"), "").unwrap();
}

/// Serving is what makes the address readable, and the switch's position comes
/// off the machine rather than off what was pressed: what a press answers with
/// is the reading, made again.
#[tokio::test]
async fn a_serve_switched_on_makes_the_address_readable() {
    let (dir, app) = app_pressing().await;
    granted(&dir);

    assert_eq!(
        remote(&app).await,
        RemoteView::Up {
            node: "workbench.tailnet-name.ts.net".to_owned(),
            serve: verkstead_render::ServeView::Off,
        },
        "nothing is served until the switch is pressed"
    );

    assert_eq!(
        press(&app, true).await,
        ServePress::Done {
            reading: RemoteView::Up {
                node: "workbench.tailnet-name.ts.net".to_owned(),
                serve: verkstead_render::ServeView::On {
                    address: "https://workbench.tailnet-name.ts.net".to_owned(),
                },
            },
        }
    );

    // And the read the pane makes on its own says the same, which is the half
    // that says the press changed the machine rather than only the answer.
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

/// And off takes the address away again — including a serve this workbench
/// never set up, because the position was read off the machine and the switch
/// turns off whatever it read.
#[tokio::test]
async fn a_serve_switched_off_takes_the_address_away() {
    let (dir, app) = app_pressing().await;

    // Set up by hand rather than by a press: the file this machine keeps its
    // serve in, written without anybody having been to the pane.
    std::fs::write(dir.path().join("serving"), "").unwrap();

    let RemoteView::Up { serve, .. } = remote(&app).await else {
        panic!("the machine should be up");
    };
    assert!(
        matches!(serve, verkstead_render::ServeView::On { .. }),
        "a serve set up by hand reads as on: {serve:?}"
    );

    assert_eq!(
        press(&app, false).await,
        ServePress::Done {
            reading: RemoteView::Up {
                node: "workbench.tailnet-name.ts.net".to_owned(),
                serve: verkstead_render::ServeView::Off,
            },
        }
    );
}

/// A serve Tailscale will not take from this user comes back as the line that
/// makes it take one — for this machine's own user, exactly as it is to be
/// typed — and a press after it has been run serves.
#[tokio::test]
async fn a_refused_serve_reads_back_the_grant_and_the_next_press_serves() {
    let (dir, app) = app_pressing().await;

    let ServePress::Ungranted { grant, trouble } = press(&app, true).await else {
        panic!("a serve refused for want of the operator grant should say so");
    };

    assert_eq!(grant, format!("sudo tailscale set --operator={WHO}"));

    // And what the machine said with it, because the grant is this build's
    // reading of a refusal and the refusal itself is the machine's.
    assert!(
        trouble.contains("Access denied"),
        "the refusal's own words should come back: {trouble}"
    );

    // Nothing was served, so the switch is still off.
    assert_eq!(
        remote(&app).await,
        RemoteView::Up {
            node: "workbench.tailnet-name.ts.net".to_owned(),
            serve: verkstead_render::ServeView::Off,
        }
    );

    // The line run in a terminal, and the same press again — which is the whole
    // of what a re-try is.
    granted(&dir);

    assert_eq!(
        press(&app, true).await,
        ServePress::Done {
            reading: RemoteView::Up {
                node: "workbench.tailnet-name.ts.net".to_owned(),
                serve: verkstead_render::ServeView::On {
                    address: "https://workbench.tailnet-name.ts.net".to_owned(),
                },
            },
        }
    );
}

/// A press on a machine with no `tailscale` at all fails as what it is, rather
/// than as a grant that would not help: there is no command to run and nothing
/// to grant it to.
#[tokio::test]
async fn a_press_with_no_tailscale_is_trouble_rather_than_a_grant() {
    let (_dir, app) = app_without_tailscale().await;

    assert!(matches!(
        press(&app, true).await,
        ServePress::Trouble { .. }
    ));
}
