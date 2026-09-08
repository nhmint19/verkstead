//! A QR code, drawn here rather than fetched.
//!
//! **Nothing is asked of anybody to draw one.** The link this encodes is the
//! Workbench Key — a workbench standing behind a secret has no business handing
//! that secret to a third party to render — and an install on a tailnet may
//! have nowhere to fetch from in the first place. So the encoder is a
//! dependency the viewer takes (`uqr`, which has none of its own) and what it
//! produces is an inline SVG: no image request, no canvas, nothing to load.
//!
//! **Black on white whatever the page is.** The code goes out to a camera
//! rather than to a reader, and a scanner expects dark modules on a light
//! ground: a QR drawn in the workbench's own colours would be one that reads on
//! half the machines it is drawn on. So the colours are stated here and the
//! surrounding pane is what the theme reaches.
//!
//! One `<path>` rather than a rectangle per module, because a version-5 code is
//! sixteen hundred of them and a path with sixteen hundred subpaths is one
//! element.

import { createMemo, type JSX } from "solid-js";
import { encode } from "uqr";

/// The quiet zone, in modules: the light border a scanner needs to find the
/// code's edges at all. Two rather than the four the specification asks for —
/// a code read off a screen is read at a size a camera can afford, and the
/// modules are worth more than the margin.
const QUIET = 2;

/// How much of the code can be lost and still read. `M` — a quarter more
/// modules than `L` for fifteen per cent of the code back, which is what a
/// phone held at an angle over a glossy screen spends.
const CORRECTION = "M" as const;

/// A link, as something to point a camera at.
export function Qr(props: {
  /// What it encodes, which is the whole of what a scan hands over.
  of: string;
  /// And what a reader is told it is, since an image of a link says nothing to
  /// anybody who is not looking at it.
  label: string;
}): JSX.Element {
  /// Encoded once per link rather than once per paint: the encoding is the
  /// expensive half and the link changes only when the key does.
  const code = createMemo(() =>
    encode(props.of, { border: QUIET, ecc: CORRECTION }),
  );

  /// The dark modules, as one path over the grid the `viewBox` is measured in.
  const modules = createMemo(() => {
    const { data } = code();
    const drawn: string[] = [];

    for (const [row, modules] of data.entries()) {
      for (const [column, dark] of modules.entries()) {
        if (dark) {
          drawn.push(`M${column} ${row}h1v1h-1z`);
        }
      }
    }

    return drawn.join("");
  });

  return (
    <svg
      role="img"
      aria-label={props.label}
      viewBox={`0 0 ${code().size} ${code().size}`}
      // Off, because the modules are a grid of exact squares: smoothing them is
      // what turns a code a camera could read into one it cannot.
      shape-rendering="crispEdges"
    >
      <rect width={code().size} height={code().size} fill="#fff" />
      <path d={modules()} fill="#000" />
    </svg>
  );
}
