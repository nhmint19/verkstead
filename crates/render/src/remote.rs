//! What the Remote access section is drawn from: what this machine's Tailscale
//! is doing, read off the machine at the moment the pane is opened.
//!
//! Nothing here is a setting. Every field is what two commands said a moment
//! ago, which is why there is no stored half to reconcile against: a tailnet
//! joined from a terminal, or a `tailscale serve` somebody set up by hand, reads
//! here exactly as one set up from this page would.
//!
//! **Three answers rather than two**, because a machine with no `tailscale` on
//! it and a machine whose daemon is not answering want different things done
//! about them — one is an install and the other is a `tailscale up` — and
//! neither of them is *serve is off*. So the state the pane narrows on says
//! which of the three it is, and only the third one carries a node name and a
//! serve state at all.
//!
//! **And a fourth that is neither**: an answer this build cannot read. The JSON
//! `tailscale status` prints is documented as subject to change between
//! releases, and Tailscale is whatever the host has rather than anything this
//! repository pins — so a shape that is not recognised says so, rather than
//! being folded into the nearest state that happens to have room for it. The
//! serve state carries the same fourth answer for the same reason: a serve
//! configuration that could not be read is *cannot tell*, never *off*, because
//! *off* is what the switch beside it would offer to turn on.
//!
//! `trouble` is what the machine said, verbatim wherever there were words to
//! take — the line `tailscale` printed on standard error is the one that names
//! the systemd unit to start, and no sentence written here would be as useful.

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

/// What this machine's Tailscale is doing.
///
/// Flat on the wire — `{"tailscale": "Up", "node": "…", "serve": {…}}` — so the
/// viewer narrows on a field rather than unwrapping a variant name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tailscale")]
#[cfg_attr(feature = "typescript", derive(TS), ts(export_to = "types.ts"))]
pub enum RemoteView {
    /// There is no `tailscale` on the PATH the server was started with. The one
    /// state whose answer is somewhere else entirely: nothing on this page can
    /// install it.
    Absent,

    /// There is, and it is not up: the daemon is not answering, or it is and
    /// this machine has not joined a tailnet.
    Down {
        /// What `tailscale` said about it — its own line where it printed one,
        /// because that is what names the service to start.
        trouble: String,
    },

    /// It answered, in a shape this build does not recognise.
    Unreadable { trouble: String },

    /// It is up, and this machine is on the tailnet as `node`.
    Up {
        /// The node's own name, as the tailnet knows it —
        /// `workbench.tailnet-name.ts.net`, with the trailing dot a DNS name
        /// carries taken off.
        node: String,

        /// And whether anything on that name is proxied to the workbench.
        serve: ServeView,
    },
}

/// Whether `tailscale serve` is putting this machine's tailnet name in front of
/// the port the workbench is served on.
///
/// The port matters: a serve of somebody else's port is not this one, and would
/// read as an address that answers with something that is not the workbench.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "serve")]
#[cfg_attr(feature = "typescript", derive(TS), ts(export_to = "types.ts"))]
pub enum ServeView {
    /// Nothing is proxied to the workbench.
    Off,

    /// Something is, and this is where it answers — the `https://….ts.net`
    /// address a phone on the tailnet is pointed at.
    On { address: String },

    /// And a serve configuration that could not be read, which is not the same
    /// as one that is empty.
    Unreadable { trouble: String },
}
