//! Reaching the workbench from a phone: what this machine's Tailscale is doing,
//! read off the machine rather than remembered.
//!
//! The Workbench Key is what makes this a section of the settings rather than a
//! recipe in the adoption docs (ADR-0015): a phone cannot reach the workbench
//! until it holds the key, and `tailscale serve --bg 8422` was a command
//! somebody ran by hand. What this module answers is the reading half of that
//! section — whether there is a Tailscale here at all, whether it is up, what
//! this node is called, and whether the tailnet name is already in front of the
//! port the workbench is served on.
//!
//! **Two commands, and nothing else.** `tailscale status --json` says whether
//! the daemon is answering and what this node is called; `tailscale serve
//! status --json` says what is proxied where. There is no Tailscale library
//! here and no socket opened by hand: the binary on the machine is the one
//! thing that is certain to speak this machine's Tailscale, whatever version it
//! happens to be.
//!
//! **Which is exactly why the reading is defensive.** Tailscale is whatever the
//! host has — nothing in this repository pins it, and the NixOS module only
//! turns the host's own on — and the JSON it prints is documented as subject to
//! change between releases. So every step that could fail has an answer of its
//! own: a binary that is not there is [`RemoteView::Absent`], a command that
//! exits non-zero is [`RemoteView::Down`] carrying whatever it printed, and a
//! shape this build cannot read is [`RemoteView::Unreadable`] rather than the
//! nearest state with room for it. The serve state is the one where that
//! matters most: *cannot tell* and *off* look the same from a distance, and the
//! switch beside it offers to turn *off* on.
//!
//! **The port is the server's own.** A serve is this workbench's when it
//! proxies to the port this process is listening on, not merely when there is a
//! serve at all: a machine already serving something else on its tailnet name
//! would otherwise read as a workbench reachable at an address that answers
//! with somebody else's page.

use std::collections::HashMap;
use std::process::Output;

use serde::Deserialize;
use tokio::process::Command;
use verkstead_render::{RemoteView, ServeView};

/// The tailscale on this machine, and the port a serve has to be pointing at
/// for it to be this workbench's.
///
/// A handle rather than a bare port, and the program held as a list, for the
/// reason [`crate::github::Gh`] holds one: what the suites put where the binary
/// goes is a shell script, and a handle is what lets them.
#[derive(Debug, Clone)]
pub struct Tailscale {
    /// What to run, and whatever stands before the arguments this module
    /// passes. `["tailscale"]` on a real machine.
    program: Vec<String>,

    /// The port the workbench is served on, which is [`crate::Config::listen`]'s
    /// rather than a constant: an install told to listen somewhere else is one
    /// whose serve has to point somewhere else too.
    port: u16,
}

/// What `tailscale status --json` says when the machine is on a tailnet. Every
/// other value of it is a machine that is not up.
const RUNNING: &str = "Running";

/// The port a `tailscale serve` answers on, which is the only one it offers:
/// HTTPS on the tailnet name. Taken off the address that is drawn, because a
/// URL naming it is a URL saying what its scheme already said.
const HTTPS: &str = "443";

impl Tailscale {
    /// The real thing: whatever `tailscale` the host has on its PATH, in front
    /// of `port`.
    pub fn on_path(port: u16) -> Tailscale {
        Tailscale::running(vec!["tailscale".to_owned()], port)
    }

    /// The same, with something else where `tailscale` goes.
    pub fn running(program: Vec<String>, port: u16) -> Tailscale {
        Tailscale { program, port }
    }

    /// What this machine's Tailscale is doing, read now.
    ///
    /// Nothing is cached. The pane is opened rarely and the answer changes
    /// whenever somebody runs `tailscale up` in a terminal, so a reading held
    /// between requests would be a pane that had to be reloaded twice to tell
    /// the truth.
    pub(crate) async fn reading(&self) -> RemoteView {
        let told = match self.run(&["status", "--json"]).await {
            Ok(told) => told,

            // The one error worth telling apart from every other: there is no
            // such program, which is a machine with no Tailscale on it rather
            // than a Tailscale that would not answer.
            Err(gone) if gone.kind() == std::io::ErrorKind::NotFound => {
                return RemoteView::Absent;
            }

            Err(trouble) => {
                return RemoteView::Down {
                    trouble: trouble.to_string(),
                };
            }
        };

        // Which is what a daemon that is not running looks like: a non-zero
        // exit, and a line on standard error naming the service to start.
        if !told.status.success() {
            return RemoteView::Down {
                trouble: complaint(&told),
            };
        }

        let status: Status = match serde_json::from_slice(&told.stdout) {
            Ok(status) => status,
            Err(trouble) => {
                return RemoteView::Unreadable {
                    trouble: format!("`tailscale status --json` could not be read: {trouble}"),
                };
            }
        };

        // A field this build does not find at all is a shape it does not know,
        // rather than a machine that is not up: the two want different things
        // said about them, and only one of them is the human's to fix.
        let Some(state) = status.backend_state else {
            return RemoteView::Unreadable {
                trouble: "`tailscale status --json` named no BackendState".to_owned(),
            };
        };

        if state != RUNNING {
            return RemoteView::Down {
                trouble: format!("the Tailscale daemon reports {state}"),
            };
        }

        // Up, and so this machine has a name on the tailnet. One that answered
        // `Running` without saying what it is called is the unreadable answer
        // again — the name is what the address is made of.
        let node = status
            .this
            .and_then(|this| this.dns_name)
            .map(|name| name.trim_end_matches('.').to_owned())
            .filter(|name| !name.is_empty());

        match node {
            None => RemoteView::Unreadable {
                trouble: "`tailscale status --json` named no node for this machine".to_owned(),
            },
            Some(node) => RemoteView::Up {
                node,
                serve: self.serving().await,
            },
        }
    }

    /// Whether anything is proxied to the workbench's port, and where it
    /// answers.
    ///
    /// Asked only of a machine that is up, because a serve configuration is
    /// kept by the daemon: with the daemon down this fails the way the status
    /// above does, and saying so twice would be one failure worded two ways.
    async fn serving(&self) -> ServeView {
        let told = match self.run(&["serve", "status", "--json"]).await {
            Ok(told) => told,
            Err(trouble) => {
                return ServeView::Unreadable {
                    trouble: trouble.to_string(),
                };
            }
        };

        if !told.status.success() {
            return ServeView::Unreadable {
                trouble: complaint(&told),
            };
        }

        proxied(&told.stdout, self.port)
    }

    /// Run `tailscale` with `arguments`, and hand back what it did.
    ///
    /// Nothing is inherited: a command run to read a machine's state has no
    /// business writing on the server's own terminal, and both streams are what
    /// this module reads its answer out of.
    async fn run(&self, arguments: &[&str]) -> std::io::Result<Output> {
        let (program, before) = self
            .program
            .split_first()
            .expect("a Tailscale is built with a program to run");

        Command::new(program)
            .args(before)
            .args(arguments)
            .stdin(std::process::Stdio::null())
            .output()
            .await
    }
}

/// What a serve configuration says about `port`.
///
/// Written apart from the running so that it can be asked the question directly:
/// what this has to get right is a JSON shape from another project, and the
/// cases worth pinning are shapes rather than machines.
fn proxied(stdout: &[u8], port: u16) -> ServeView {
    // `null` is what a machine nobody has ever run `tailscale serve` on prints,
    // and `{}` is what one that has had every serve taken off again prints. Both
    // are off rather than unreadable: an empty configuration is a configuration.
    let told: Option<serde_json::Map<String, serde_json::Value>> =
        match serde_json::from_slice(stdout) {
            Ok(told) => told,
            Err(trouble) => {
                return ServeView::Unreadable {
                    trouble: format!(
                        "`tailscale serve status --json` could not be read: {trouble}"
                    ),
                };
            }
        };

    let Some(told) = told.filter(|told| !told.is_empty()) else {
        return ServeView::Off;
    };

    // A configuration holding something and holding no `Web` at all is this
    // build reading a shape it does not know — a serve of a port is a `Web`
    // entry, and a release that renamed the section would otherwise read as a
    // machine serving nothing.
    let Some(web) = told.get("Web") else {
        return ServeView::Unreadable {
            trouble: "the serve configuration named no Web section".to_owned(),
        };
    };

    let hosts: HashMap<String, Host> = match serde_json::from_value(web.clone()) {
        Ok(hosts) => hosts,
        Err(trouble) => {
            return ServeView::Unreadable {
                trouble: format!(
                    "the serve configuration's Web section could not be read: {trouble}"
                ),
            };
        }
    };

    let served = hosts.iter().find(|(_, host)| {
        host.handlers
            .values()
            .filter_map(|handler| handler.proxy.as_deref())
            .any(|proxy| proxies_to(proxy, port))
    });

    match served {
        None => ServeView::Off,
        Some((host, _)) => ServeView::On {
            address: address_of(host),
        },
    }
}

/// Whether a handler's proxy target is the workbench's own port.
///
/// The port and nothing else: what stands in front of it is a loopback address
/// written whichever way the machine writes one, and a serve pointed at this
/// port is this workbench's however it spells the host.
fn proxies_to(proxy: &str, port: u16) -> bool {
    let authority = proxy.split_once("://").map_or(proxy, |(_, rest)| rest);
    let authority = authority.split(['/', '?']).next().unwrap_or(authority);

    authority
        .rsplit_once(':')
        .is_some_and(|(_, named)| named == port.to_string())
}

/// The address a served host answers on, as something to point a browser at.
///
/// The key of a `Web` section is `host:port`, and the port is the one a serve
/// offers — so it comes off the address rather than being written into it. A key
/// on any other port keeps it, there being nothing else to say about one.
fn address_of(host: &str) -> String {
    match host.rsplit_once(':') {
        Some((name, HTTPS)) => format!("https://{name}"),
        _ => format!("https://{host}"),
    }
}

/// What a command that failed said about it: its first line on standard error,
/// which is where `tailscale` puts the sentence naming the service to start.
///
/// Its own words rather than a sentence written here, because the useful half of
/// them is the machine's — `sudo systemctl start tailscaled` is not something
/// this build could have known to say.
fn complaint(told: &Output) -> String {
    String::from_utf8_lossy(&told.stderr)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| format!("tailscale exited with {}", told.status))
}

/// The half of `tailscale status --json` this reads.
///
/// Every field optional, because every one of them is another project's to
/// rename: what is missing is answered as an answer this build cannot read
/// rather than as a machine in some particular state.
#[derive(Debug, Deserialize)]
struct Status {
    #[serde(rename = "BackendState")]
    backend_state: Option<String>,

    #[serde(rename = "Self")]
    this: Option<Node>,
}

/// And the half of the node it names: what this machine is called on the
/// tailnet.
#[derive(Debug, Deserialize)]
struct Node {
    /// A DNS name, so it arrives with the trailing dot one carries.
    #[serde(rename = "DNSName")]
    dns_name: Option<String>,
}

/// One host of a serve configuration's `Web` section, keyed by `host:port`.
#[derive(Debug, Deserialize)]
struct Host {
    #[serde(rename = "Handlers", default)]
    handlers: HashMap<String, Handler>,
}

/// And one handler under it: a path, and what is behind it.
#[derive(Debug, Deserialize)]
struct Handler {
    /// Where it proxies to, absent on a handler that serves something else —
    /// a directory, or text written into the configuration.
    #[serde(rename = "Proxy")]
    proxy: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What `tailscale serve status --json` prints on a machine serving the
    /// workbench: HTTPS on the node's name, proxied to the loopback port.
    const SERVED: &str = r#"{
      "TCP": { "443": { "HTTPS": true } },
      "Web": {
        "workbench.tailnet-name.ts.net:443": {
          "Handlers": { "/": { "Proxy": "http://127.0.0.1:8422" } }
        }
      }
    }"#;

    /// A machine that has never been served reads as off, and so does one every
    /// serve has been taken off again.
    #[test]
    fn nothing_configured_is_off() {
        assert_eq!(proxied(b"null\n", 8422), ServeView::Off);
        assert_eq!(proxied(b"{}\n", 8422), ServeView::Off);
    }

    /// And a serve of the workbench's port is on, at the address the tailnet
    /// reaches it by — without the port, which is the only one a serve offers.
    #[test]
    fn a_serve_of_the_workbenchs_port_names_the_address() {
        assert_eq!(
            proxied(SERVED.as_bytes(), 8422),
            ServeView::On {
                address: "https://workbench.tailnet-name.ts.net".to_owned()
            }
        );
    }

    /// A serve of somebody else's port is not this workbench's. The address it
    /// answers on would answer with something that is not the workbench, so
    /// naming it here would be pointing a phone at the wrong page.
    #[test]
    fn a_serve_of_another_port_is_off() {
        assert_eq!(proxied(SERVED.as_bytes(), 9000), ServeView::Off);
    }

    /// A serve configuration this build cannot read says so, rather than
    /// reading as a machine serving nothing: *off* is what the switch beside it
    /// offers to turn on.
    #[test]
    fn a_shape_this_build_does_not_know_is_not_off() {
        assert!(matches!(
            proxied(b"not json at all", 8422),
            ServeView::Unreadable { .. }
        ));

        assert!(matches!(
            proxied(br#"{"Sites": {"workbench:443": {}}}"#, 8422),
            ServeView::Unreadable { .. }
        ));
    }

    /// A proxy target is matched on its port, however the host in front of it is
    /// written — a serve set up by hand names the loopback whichever way the
    /// person setting it up did.
    #[test]
    fn a_proxy_is_matched_on_its_port() {
        assert!(proxies_to("http://127.0.0.1:8422", 8422));
        assert!(proxies_to("127.0.0.1:8422", 8422));
        assert!(proxies_to("http://localhost:8422/", 8422));
        assert!(proxies_to("http://[::1]:8422", 8422));

        assert!(!proxies_to("http://127.0.0.1:8423", 8422));
        assert!(!proxies_to("http://127.0.0.1", 8422));
    }

    /// The failing command's own line is what comes back, because it is the one
    /// that names the service to start.
    #[test]
    fn a_refusal_is_reported_in_the_machines_own_words() {
        let told = Output {
            status: Default::default(),
            stdout: Vec::new(),
            stderr: b"\nfailed to connect to local tailscaled; it doesn't appear to be running\n"
                .to_vec(),
        };

        assert_eq!(
            complaint(&told),
            "failed to connect to local tailscaled; it doesn't appear to be running"
        );
    }
}
