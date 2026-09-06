//! A path as the filesystem has it: `..` taken out and every symlink followed.
//!
//! What a Repo's path and an Agent Profile's account are put through before
//! anything is done with them, and what is stored afterwards. A path as it was
//! written may name several directories — `/home/ada/src/../src` and a symlink
//! into somebody else's tree both name one — and the resolved path names the
//! directory a session will actually be run against.
//!
//! Two questions and no more. Whether a path was *allowed* was a third once,
//! asked of the Watched Paths, and is asked nowhere now: what keeps a session to
//! its own Conversation is the Sandbox, composed from the Repo and the Profile
//! that Conversation names.

use std::path::{Path, PathBuf};

/// What resolving a path made of it.
///
/// The two refusals are named apart rather than lumped together because each is
/// a different sentence to put in front of the human: a path typed relative is
/// one to write out in full, and a path with nothing at it is a directory to go
/// and make.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Resolved {
    /// Where it really is. This is the path to work with from here on.
    At(PathBuf),

    /// Relative. Nothing here resolves one — the directory the server happens to
    /// be running in is not something a path should mean, a service unit and a
    /// checkout starting in different places.
    NotAbsolute,

    /// Nothing is there to resolve.
    Missing,
}

/// `path` as the filesystem has it, or what is wrong with it.
///
/// Blocking: resolving a path is a filesystem read.
pub(crate) fn resolve(path: &Path) -> Resolved {
    if !path.is_absolute() {
        return Resolved::NotAbsolute;
    }

    match path.canonicalize() {
        Ok(real) => Resolved::At(real),
        Err(_) => Resolved::Missing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_directory_resolves_to_itself() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(
            resolve(dir.path()),
            Resolved::At(dir.path().canonicalize().unwrap())
        );
    }

    /// The point of resolving: `..` is taken out, so what is stored names one
    /// directory however it was spelled.
    #[test]
    fn a_path_that_climbs_and_comes_back_resolves_to_where_it_lands() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("verkstead");
        std::fs::create_dir(&dir).unwrap();

        let roundabout = dir.join("..").join("verkstead");

        assert_eq!(
            resolve(&roundabout),
            Resolved::At(dir.canonicalize().unwrap())
        );
    }

    /// And so are symlinks: what comes back is where the link points, which is
    /// the directory anything done with it would touch.
    #[cfg(unix)]
    #[test]
    fn a_symlink_resolves_to_what_it_points_at() {
        let root = tempfile::tempdir().unwrap();
        let elsewhere = root.path().join("elsewhere");
        let link = root.path().join("link");
        std::fs::create_dir(&elsewhere).unwrap();
        std::os::unix::fs::symlink(&elsewhere, &link).unwrap();

        assert_eq!(
            resolve(&link),
            Resolved::At(elsewhere.canonicalize().unwrap())
        );
    }

    #[test]
    fn a_relative_path_is_refused_without_being_resolved() {
        assert_eq!(resolve(Path::new("verkstead")), Resolved::NotAbsolute);
    }

    #[test]
    fn a_path_with_nothing_at_it_is_missing() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(resolve(&dir.path().join("never-made")), Resolved::Missing);
    }
}
