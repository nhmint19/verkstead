//! What the human has done with a banner: the one flag that says the Remote
//! access banner has been read and is not to be drawn again.
//!
//! The banner stands on a Conversation page above the Timeline, at every
//! grilling start, while the first Question Set is being prepared — the one
//! moment there is nothing to do at the desk — and it points at the Remote
//! access section, which is where a phone is let in from. So it is drawn again
//! and again until somebody says they have read it.
//!
//! **Which is a fact about the human rather than about the browser they said it
//! in.** The whole of what the banner is for is somebody at a desk about to
//! pick up a phone, and a dismissal kept in that desk's own storage would meet
//! them again on the very device it had just pointed them at. So it is here,
//! read back off the server on every load, exactly as the archived switch's
//! position is — see [`super::showing_archived`], whose one-row table this one
//! is written beside.
//!
//! One direction, unlike that switch: the banner is dismissed and stays
//! dismissed. There is nothing on the page that puts it back, because what it
//! has to say is said once — and a `DELETE` here is a database somebody has
//! edited by hand, which is answer enough for a flag nothing else turns on.

use anyhow::{Context, Result};
use sqlx::SqlitePool;

/// The table the dismissal's one row lives in.
///
/// One row or none, the presence of it being the whole of the flag: a column
/// holding a `0` would be a second way to say what an empty table already says.
pub(crate) async fn apply_schema(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS dismissed_remote_banner (
             only_row INTEGER PRIMARY KEY CHECK (only_row = 0)
         ) STRICT",
    )
    .execute(pool)
    .await
    .context("creating the dismissed_remote_banner table")?;

    Ok(())
}

/// Whether the Remote access banner has been dismissed.
///
/// What every Conversation page asks on load, which is what makes one press on
/// any device the end of it everywhere.
pub async fn remote_banner_dismissed(pool: &SqlitePool) -> Result<bool> {
    let row: Option<(i64,)> = sqlx::query_as("SELECT only_row FROM dismissed_remote_banner")
        .fetch_optional(pool)
        .await
        .context("reading whether the Remote access banner has been dismissed")?;

    Ok(row.is_some())
}

/// And say that it has been, which is one row written.
///
/// Idempotent: the human's press says *done with this*, and a second press
/// saying it again is not a thing to refuse.
pub async fn dismiss_remote_banner(pool: &SqlitePool) -> Result<()> {
    sqlx::query("INSERT INTO dismissed_remote_banner (only_row) VALUES (0) ON CONFLICT DO NOTHING")
        .execute(pool)
        .await
        .context("dismissing the Remote access banner")?;

    Ok(())
}
