//! Reaching the workbench from a phone: what this machine's Tailscale is doing,
//! read off the machine rather than remembered.
//!
//! The Workbench Key is what makes this a section of the settings rather than a
//! recipe in the adoption docs (ADR-0015): a phone cannot reach the workbench
//! until it holds the key, and `tailscale serve --bg 8422` was a command
//! somebody ran by hand. What this module answers is that section — whether
//! there is a Tailscale here at all, whether it is up, what this node is
//! called, whether the tailnet name is already in front of the port the
//! workbench is served on, and the one press that puts it there.
//!
//! **Three commands, and nothing else.** `tailscale status --json` says whether
//! the daemon is answering and what this node is called; `tailscale serve
//! status --json` says what is proxied where; and `tailscale serve` is the
//! switch. There is no Tailscale library here and no socket opened by hand: the
//! binary on the machine is the one thing that is certain to speak this
//! machine's Tailscale, whatever version it happens to be.
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
//!
//! **The press is refused before it is run.** `tailscale serve` from a process
//! that is neither root nor the tailnet's operator is denied by the daemon, and
//! there is nothing the server can do about that: it runs unprivileged, and a
//! daemon that could make itself the operator would be a daemon that could do
//! anything. So a denied press hands back the line that grants it —
//! `sudo tailscale set --operator=<user>`, for this machine's own user — and
//! the next press is the re-try. The desktop app runs the same line through the
//! platform's graphical sudo; the daemon on its own only shows it.

use std::collections::HashMap;
use std::process::Output;

use serde::Deserialize;
use tokio::process::Command;
use verkstead_render::{RemoteView, ServePress, ServeView};

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

    /// Who this process is, for the one sentence that has to name them: the
    /// operator grant. Read once, at startup, because it is what the machine's
    /// own user is called and nothing about a request changes it.
    user: String,
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
        Tailscale {
            program,
            port,
            user: this_user(),
        }
    }

    /// The same again, with the machine's own user stated rather than read off
    /// the environment.
    ///
    /// For the suites, which have the operator grant to check: the command it
    /// hands back names a user, and a test whose expected line came out of
    /// whoever happens to be running `cargo test` would be a test of nothing.
    pub fn as_user(self, user: String) -> Tailscale {
        Tailscale { user, ..self }
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

    /// Put the serve where the switch was pressed to, and read the machine
    /// again.
    ///
    /// One command each way — `tailscale serve --bg <port>` on, and
    /// `tailscale serve --https=443 off` off — and then the reading, because
    /// the switch's position comes off the machine rather than off what this
    /// press meant to do. A serve that would not go on comes back as a switch
    /// still off, which is the truth about the machine and the only thing worth
    /// drawing.
    ///
    /// The one answer that is not the reading is the operator grant: Tailscale
    /// refuses a serve from a process that is neither root nor the tailnet's
    /// operator, and nothing this server can do lifts that. So the refusal
    /// carries the line that does — for this machine's own user — and the next
    /// press is the re-try.
    pub(crate) async fn press(&self, on: bool) -> ServePress {
        let port = self.port.to_string();
        let off = format!("--https={HTTPS}");

        let arguments: Vec<&str> = match on {
            true => vec!["serve", "--bg", &port],
            false => vec!["serve", &off, "off"],
        };

        let told = match self.run(&arguments).await {
            Ok(told) => told,
            Err(trouble) => {
                return ServePress::Trouble {
                    trouble: trouble.to_string(),
                };
            }
        };

        if !told.status.success() {
            let trouble = complaint(&told);

            return match ungranted(&trouble) {
                true => ServePress::Ungranted {
                    grant: grant(&self.user),
                    trouble,
                },
                false => ServePress::Trouble { trouble },
            };
        }

        ServePress::Done {
            reading: self.reading().await,
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

/// Whether what `tailscale` printed is it refusing to take the change from this
/// user, rather than failing to make it.
///
/// Read off the words, because there is nothing else to read it off: the
/// refusal is the daemon's, it comes back through the CLI as an exit status and
/// a line, and asking who the operator is would mean a command Tailscale
/// documents as unstable. So this matches on what the refusal is called —
/// *access denied*, *permission denied*, or the word *operator* itself, whoever
/// happened to put it there.
///
/// Wrong either way costs the same small thing and nothing more. A refusal not
/// recognised is shown as what the machine said, which is what the human needed
/// anyway; something else mistaken for one is that line with a grant offered
/// under it, and running the grant is harmless on a machine that did not need
/// it.
fn ungranted(trouble: &str) -> bool {
    let said = trouble.to_lowercase();

    ["access denied", "permission denied", "operator"]
        .iter()
        .any(|refusal| said.contains(refusal))
}

/// The line that makes this machine's user Tailscale's operator, which is what
/// lets a serve be set up by anything but root.
///
/// Handed over rather than run. Nothing here escalates: the server has no
/// privilege to raise, and a daemon that quietly acquired one would be a worse
/// thing than a switch that has to be pressed twice.
fn grant(user: &str) -> String {
    format!("sudo tailscale set --operator={user}")
}

/// What this machine's own user is called, for the grant above to name.
///
/// The environment rather than the passwd database: systemd sets `USER` for
/// every service it starts, a shell sets it for everything run from one, and
/// reading a user out of libc would be a dependency and a platform's worth of
/// conditional compilation for one string.
///
/// Where neither is set there is still a command to hand over — one that works
/// out the name in the shell it is pasted into, which is the same line with the
/// same effect and one more thing for the human to trust.
fn this_user() -> String {
    ["USER", "LOGNAME"]
        .iter()
        .filter_map(|named| std::env::var(named).ok())
        .find(|user| !user.is_empty())
        .unwrap_or_else(|| "$(id -un)".to_owned())
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

    /// A refusal is told apart from a failure, because only one of them has a
    /// command under it: the grant is what makes the switch work on the next
    /// press, and offering it under a daemon that is simply not running would
    /// be sending somebody to a terminal for nothing.
    #[test]
    fn a_refused_serve_is_told_apart_from_a_failed_one() {
        assert!(ungranted("Access denied: serve config denied"));
        assert!(ungranted(
            "access denied: must be operator or root to modify serve config"
        ));
        assert!(ungranted("error setting serve config: permission denied"));

        assert!(!ungranted(
            "failed to connect to local tailscaled; it doesn't appear to be running"
        ));
        assert!(!ungranted(
            "HTTPS must be enabled in the admin console to use serve"
        ));
    }

    /// And the grant names the machine's own user, because that is the whole of
    /// what makes it a line somebody can paste.
    #[test]
    fn the_grant_names_this_machines_user() {
        assert_eq!(grant("ada"), "sudo tailscale set --operator=ada");
    }
}
