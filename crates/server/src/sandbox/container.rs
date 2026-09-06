//! The identity a Windows session runs under: an AppContainer of one
//! Conversation's own, and what says a process is really inside one.
//!
//! **What an AppContainer is, for the purposes of this file.** It is not a
//! wrapper a process is started under and not a namespace it is put into: it is
//! an *identity*, a SID the operating system makes and keeps, and a process
//! started with it on its token reaches only what that SID has been granted.
//! Which is why there is a profile to create here and access-control entries to
//! write in [`super::granting`] — the boundary on this platform is the reach of a
//! name rather than a wall around a process (ADR-0014).
//!
//! **One profile per Conversation**, named from the Data Directory and the
//! Conversation's id. The Data Directory half is the same fingerprint the
//! server's pipe is named from — see [`crate::pipe`] — so two Verksteads on one
//! machine keep their containers apart the way they keep their pipes apart, and
//! the Conversation half is what makes one Conversation's session refused
//! another's Worktree.
//!
//! **The capability is the internet client and nothing else** (ADR-0014, Q16).
//! A session has to reach GitHub, a registry and the model's API; it has no
//! business on this machine's own network, and the probe found that it has none
//! — a connection from inside to `127.0.0.1` and to the machine's own address
//! both time out. What it asks Verkstead through instead is the named pipe.
//!
//! **Which is why a container tells the pipe about itself as it is made.** The
//! server opened its pipe at startup, before this profile existed, and the one
//! thing a session inside here can reach Verkstead by is that pipe granting
//! this identity — so the SID goes into [`crate::pipe::Grants`] the moment it
//! is read back, and comes out of it again when the profile goes. A pipe that
//! will not take it refuses the container, which refuses the session: a session
//! that cannot ask is not one to start.
//!
//! **What creates one is what deletes it, and the entries go with it.** A
//! [`Container`] takes back every access-control entry written for its identity
//! and then deletes its profile as it is dropped, so a boundary proved by
//! attempting it leaves the machine as it found it. The order is that way round
//! deliberately: the SID is what names an entry, so a profile deleted first
//! would leave entries standing on the human's own directories as a number
//! nothing can resolve.
//!
//! **Its life is the Conversation's, rather than a session's** (ADR-0014, Q14).
//! A Conversation can have a session and a Conversation Terminal going at once
//! and both are inside the same profile, and the session after those is inside
//! it too: so what [`Container::for_conversation`] hands out is shared, and what
//! holds it is this module rather than whoever is running inside. Granted at the
//! first session, and let go of at exactly two moments — the Conversation being
//! closed, which takes its Worktree in the same breath, and the sweep a server
//! starts with. Both arrive here as [`taken_back`].
//!
//! **Which is why what was written is also written down.** A held profile is a
//! fact about this process and its entries are a fact about the human's disk, so
//! a server that died is one that left a boundary standing that nothing in
//! memory describes any more. Every container of a Conversation's therefore
//! keeps a record under the Data Directory — see
//! [`super::granting::remembering`] — written before anything is granted for it,
//! and it is that record the sweep takes a crashed-over container back by. A
//! container that cannot be written down is refused, for the reason a pipe that
//! will not grant it is: what cannot be taken away afterwards should not be made
//! now.

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::ptr;
use std::sync::{Arc, LazyLock, Mutex};

use windows_sys::Win32::Foundation::{HANDLE, LocalFree};
use windows_sys::Win32::Security::Authorization::{ConvertSidToStringSidW, ConvertStringSidToSidW};
use windows_sys::Win32::Security::Isolation::{
    CreateAppContainerProfile, DeleteAppContainerProfile,
};
use windows_sys::Win32::Security::{
    EqualSid, FreeSid, GetLengthSid, GetTokenInformation, PSID, SID_AND_ATTRIBUTES,
    TOKEN_APPCONTAINER_INFORMATION, TOKEN_QUERY, TokenAppContainerSid,
};
use windows_sys::Win32::System::Threading::{
    OpenProcess, OpenProcessToken, PROCESS_QUERY_LIMITED_INFORMATION,
};

use super::granting::{Entry, remembering, writing};
use super::starting::{Handle, wide};

/// Every profile this process is holding, by the name it was made under.
///
/// **A profile is one machine-wide thing and this is what keeps it one.** Two
/// things run inside a Conversation's container — a grilling session and the
/// Conversation Terminal beside it — and each of them asks for the container as
/// it starts. Made twice, the second would be refused the name the first is
/// holding; deleted twice, the first to end would take the profile out from
/// under the one still running. So what is handed out is shared, and this is
/// where the sharing is done.
///
/// **And the hold is a strong one, which is the Conversation's lifetime said in
/// code.** A profile outlives every session that runs inside it — the next one
/// is inside the same profile, and its entries are what makes the Worktree
/// reachable at all — so what lets go of it is not a session ending but
/// [`taken_back`], which is the close and the sweep. Held weakly, a
/// Conversation's boundary would be built and taken down around every session
/// it runs, which is a directory tree walked at each start for entries that were
/// already there.
static PROFILES: LazyLock<Mutex<HashMap<String, Arc<Container>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// The one capability a session's container is created with: the internet
/// client.
///
/// Written as a SID rather than looked up by name, because that is what it is:
/// `S-1-15-3-1` is `internetClient` on every Windows there is.
const INTERNET_CLIENT: &str = "S-1-15-3-1";

/// What an enabled group is, in the attributes half of a capability. Win32's
/// own number, which the bindings do not carry.
const SE_GROUP_ENABLED: u32 = 0x0000_0004;

/// One AppContainer, made and held.
///
/// What travels to a rendering is [`Container::sid`] — see
/// [`super::Rendering::inside`], which is the whole of how the identity crosses
/// the seam — and what is held here is the profile itself, which goes when this
/// does.
#[derive(Debug)]
pub struct Container {
    /// What the profile is called on this machine, which is what deletes it.
    name: String,

    /// And the identity it is, as a SID is written down: what a process is
    /// started with, and what a directory is granted to.
    sid: String,

    /// Every access-control entry written on a real directory for that
    /// identity, so that every one of them can be taken back.
    ///
    /// Kept here rather than by whoever wrote them, because the entries and the
    /// profile are one thing: an entry names this SID and nothing else does, so
    /// the moment the profile goes is the moment an entry left behind stops
    /// meaning anything. Which is also why this is the whole list rather than
    /// the last session's — a Conversation's second session writes the entries
    /// its own description says, and what has to come off at the end is all of
    /// them.
    granted: Mutex<Vec<Entry>>,

    /// And where this container is written down, where it is one of a
    /// Conversation's rather than one the suite made by name.
    ///
    /// `None` for a container made under a name said outright — see
    /// [`Container::named`] — which is nobody's Conversation and has nothing to
    /// be swept by: what makes a record worth keeping is a *later* server
    /// needing to take this away, and a container that belongs to no
    /// Conversation is one no later server would know what to do with.
    kept: Option<Kept>,
}

/// Where a Conversation's container is written down: the Data Directory the
/// record goes under, and whose record it is.
///
/// The pair rather than the composed path, because both halves are wanted
/// separately — the record is read and written by Conversation, and the sweep
/// reads every one there is under the same directory.
#[derive(Debug, Clone)]
struct Kept {
    data_dir: std::path::PathBuf,
    conversation: i64,
}

impl Container {
    /// The container the sessions of `conversation` run in, under the Data
    /// Directory at `data_dir` — made at the first of them and shared with
    /// every one after it.
    ///
    /// The name is both halves because both are needed: the Data Directory so
    /// that two Verksteads on one machine are two sets of containers, and the
    /// Conversation so that what one Conversation's session may reach is not
    /// what the next one's may — see this module's own documentation.
    pub fn for_conversation(data_dir: &Path, conversation: i64) -> io::Result<Arc<Container>> {
        held(data_dir, conversation)
    }

    /// The entries written for this identity, remembered so that they can be
    /// taken back when it goes — here and on the disk both.
    ///
    /// Added to rather than replaced: a Conversation's second session describes
    /// its own surface, and an entry written by the first is still an entry on
    /// somebody's directory.
    ///
    /// **Called before the entries are written on the machine**, and it can
    /// refuse. What this hands back a failure for is a record that would not
    /// write, which is a boundary no later server could take back: so the
    /// caller refuses the session, and nothing has been granted yet to leave
    /// behind. Remembering an entry the write below then failed on is the safe
    /// side of the same order — taking back an entry that is not there is
    /// nothing at all.
    pub(crate) fn wrote(&self, entries: Vec<Entry>) -> io::Result<()> {
        {
            let mut granted = self.granted.lock().unwrap_or_else(|held| held.into_inner());

            for entry in entries {
                if !granted.contains(&entry) {
                    granted.push(entry);
                }
            }
        }

        self.remember()
    }

    /// This container as it is written down, where it is one of a
    /// Conversation's — and nothing to do at all where it is not.
    fn remember(&self) -> io::Result<()> {
        let Some(kept) = &self.kept else {
            return Ok(());
        };

        remembering::wrote(
            &kept.data_dir,
            &remembering::Remembered {
                conversation: kept.conversation,
                name: self.name.clone(),
                sid: self.sid.clone(),
                entries: self
                    .granted
                    .lock()
                    .unwrap_or_else(|held| held.into_inner())
                    .clone(),
            },
        )
    }

    /// And one under a name said outright, which is what the suite makes.
    ///
    /// Nobody's Conversation, and so nothing written down: what gives a
    /// container a record is [`held`], which is where a Conversation's own is
    /// made.
    ///
    /// **A name already taken is an error rather than the profile that is
    /// there.** A container Verkstead did not make is one whose capabilities
    /// and whose grants are somebody else's, and a session started inside it
    /// would be a session behind a boundary nothing here described (ADR-0014,
    /// Q18): what cannot be made refuses the session rather than falling back.
    pub fn named(name: &str) -> io::Result<Container> {
        Container::made(name, None)
    }

    /// The same profile, written down as it is made where it is a
    /// Conversation's.
    ///
    /// **Everything that can refuse happens before there is a [`Container`] to
    /// let go of**, which is not a nicety of shape: this is called with the map
    /// of profiles held — see [`held`] — and a `Container` dropped there would
    /// be a [`Container::drop`] taking that same lock. So each refusal below
    /// deletes the profile itself and hands back a reason, and the record is
    /// written last of the three, when there is nothing after it that could
    /// fail.
    fn made(name: &str, kept: Option<Kept>) -> io::Result<Container> {
        // Held in a value of its own, which is not a spelling to tidy: a
        // temporary here would be a SID given back before the call below is
        // made, and what that call would read is whatever the machine put in
        // its place.
        let internet = Sid::of(INTERNET_CLIENT)?;

        let capability = SID_AND_ATTRIBUTES {
            Sid: internet.as_psid(),
            Attributes: SE_GROUP_ENABLED,
        };

        let name_w = wide(OsStr::new(name));
        let mut sid: PSID = ptr::null_mut();

        // The display name and the description are the profile's own name said
        // again: nothing shows them to anybody — an AppContainer of Verkstead's
        // is not an installed application — and a name is more use than a blank
        // to whoever goes looking at what is on their machine.
        let made = unsafe {
            CreateAppContainerProfile(
                name_w.as_ptr(),
                name_w.as_ptr(),
                name_w.as_ptr(),
                &capability,
                1,
                &mut sid,
            )
        };

        if made < 0 {
            return Err(io::Error::other(format!(
                "the AppContainer {name} could not be made: CreateAppContainerProfile said {}",
                said(made)
            )));
        }

        let held = Made(sid);

        let Some(sid) = written(held.0) else {
            return Err(io::Error::other(format!(
                "the AppContainer {name} was made and its SID could not be read back: {}",
                io::Error::last_os_error()
            )));
        };

        // And the pipe told, before anything is inside the container to ask
        // through it. A session cannot dial the loopback from in there — which
        // is the whole reason there is a pipe (ADR-0014) — so a container the
        // pipe was never told about is one whose session can reach Verkstead by
        // no route at all, and a session that cannot ask is refused rather than
        // started. The profile goes with the refusal: what is left behind
        // otherwise is a name the next attempt would be turned away by.
        if let Err(refused) = crate::pipe::Grants::of_this_process().to(&sid) {
            unsafe { DeleteAppContainerProfile(name_w.as_ptr()) };

            return Err(io::Error::other(format!(
                "the AppContainer {name} was made and the pipe a session asks Verkstead through \
                 could not be told to grant it, so it was deleted again: {refused}"
            )));
        }

        // And written down, with nothing granted for it yet. Before rather than
        // after the first grant, which is the whole point of it: what a record
        // is for is a *later* server taking this away, and the moment there is
        // anything to take away is the moment after this one. A record that will
        // not write is a boundary nothing could ever clear, so the container is
        // refused and what was made of it goes with the refusal.
        if let Some(kept) = &kept {
            let remembered = remembering::Remembered {
                conversation: kept.conversation,
                name: name.to_owned(),
                sid: sid.clone(),
                entries: Vec::new(),
            };

            if let Err(refused) = remembering::wrote(&kept.data_dir, &remembered) {
                crate::pipe::Grants::of_this_process().no_longer(&sid);

                unsafe { DeleteAppContainerProfile(name_w.as_ptr()) };

                return Err(io::Error::other(format!(
                    "the AppContainer {name} was made and could not be written down under the \
                     Data Directory, so nothing would ever have taken it off this machine \
                     again and it was deleted: {refused}"
                )));
            }
        }

        Ok(Container {
            name: name.to_owned(),
            sid,
            granted: Mutex::new(Vec::new()),
            kept,
        })
    }

    /// What the profile is called on this machine.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The identity itself: what a process is started inside and what a
    /// directory is granted to.
    pub fn sid(&self) -> &str {
        &self.sid
    }
}

impl Drop for Container {
    /// The entries go, and then the profile goes with whatever was holding it.
    ///
    /// That order and not the other one: an entry names this identity by its
    /// SID, so a profile deleted first would leave every directory it was
    /// granted on carrying a number nothing on the machine can resolve — which
    /// is what the probe was careful about too, on the human's own directories.
    ///
    /// **What runs this is [`taken_back`]**, ordinarily: a Conversation's
    /// container is held by this module for as long as the Conversation is
    /// working, so the last hold to go is the close or the sweep letting go of
    /// it. Where a session is still running at that moment the hold is the
    /// session's for a little longer, and this runs when its relay lets go —
    /// which is the right order rather than a race: the boundary a running
    /// session is behind is not one to take down around it.
    ///
    /// Nothing is reported about the profile itself: one that will not delete
    /// is not something the code that let go of it can do anything about, and
    /// what finds it again is the sweep the next server starts with.
    fn drop(&mut self) {
        let granted =
            std::mem::take(&mut *self.granted.lock().unwrap_or_else(|held| held.into_inner()));

        let profiles = PROFILES.lock().unwrap_or_else(|held| held.into_inner());

        // A live hold under this name is a container made *after* this one, in
        // the moment between this one's hold being taken out of the map and
        // this running — see [`held`], which makes a fresh profile for a name
        // nothing is holding. Its profile is not this one's to delete, and the
        // entries this wrote are the same SID's, so they go to it rather than
        // being taken back from under it. Nor is the pipe told to stop granting
        // the identity: it is that container's identity as much as this one's,
        // and what is running inside it is asking through the pipe right now.
        if let Some(taken) = profiles.get(&self.name).cloned() {
            // The record is that container's too, and rewriting it is how the
            // entries this hands over reach the disk. Nothing is done about a
            // refusal: what this is holding has already been handed on, and the
            // container that now has it is one a caller could refuse for.
            if let Err(error) = taken.wrote(granted) {
                tracing::warn!(
                    name = self.name,
                    error = ?error,
                    "the entries of a container that has gone were handed to the one that \
                     took its name and could not be written down with it"
                );
            }

            return;
        }

        // The pipe before either of the two below, and for their reason: an
        // identity still granted on a pipe whose profile has been deleted is an
        // entry naming nobody, which is exactly what an access-control entry
        // left behind on a directory would be.
        //
        // All of it while the map is held, so that a caller arriving in the
        // meantime waits and then makes a profile of its own rather than
        // finding this one part-way out. What that costs is a session start
        // held up by another Conversation's ending, for as long as it takes to
        // walk back the trees this was granted.
        given_back(
            &self.name,
            &self.sid,
            &granted,
            self.kept
                .as_ref()
                .map(|kept| (kept.data_dir.as_path(), kept.conversation)),
        );
    }
}

/// One container taken off the machine: the pipe told, the entries taken off
/// every directory they were written on, the profile deleted, and the record of
/// it removed.
///
/// The one place any of that happens, because the order is the whole of it and
/// there are two ways in: a container this process is holding, which arrives
/// through [`Container::drop`], and one a server that has gone left behind,
/// which arrives off its record through [`taken_back`]. Both are the same four
/// things in the same order.
///
/// `record` is where this was written down and whose it is, where it was written
/// down at all.
fn given_back(name: &str, sid: &str, granted: &[Entry], record: Option<(&Path, i64)>) {
    crate::pipe::Grants::of_this_process().no_longer(sid);

    // The entries first and the profile after them, which is the order the
    // probe was careful about on the human's own directories: the SID is what
    // names an entry, and a profile deleted first leaves every one of them
    // standing as a number nothing on the machine can resolve.
    writing::strip(granted, sid);

    unsafe { DeleteAppContainerProfile(wide(OsStr::new(name)).as_ptr()) };

    // And the record last of all, for the same reason said one step out: a
    // record is what the next sweep would take this back by, so it goes once
    // there is nothing left to take back.
    if let Some((data_dir, conversation)) = record {
        remembering::forget(data_dir, conversation);
    }
}

/// The container of `conversation`, let go of: its entries off the human's
/// directories, its profile off the machine and its record with them.
///
/// **The two ways a Conversation's boundary ends**, and both of them come
/// through here: the close, which takes the profile in the same breath as the
/// Worktree, and the sweep a server starts with — see [`crate::containers`],
/// which is where both are decided.
///
/// **A container this process is holding is let go of, and one it is not is
/// taken back off its record.** The second is what a crash leaves: the profile
/// and its entries are on the machine and nothing in memory knows about either,
/// so what says which directories carry an entry is what the server that wrote
/// them wrote down — see [`super::granting::remembering`].
///
/// **Nothing is refused and nothing comes back.** What could be done about a
/// profile that will not delete is what the next sweep will do about it anyway,
/// and a close that failed for it would be a Conversation nothing can ever end.
pub fn taken_back(data_dir: &Path, conversation: i64) {
    let name = profile(data_dir, conversation);

    // Taken out of the map under the lock and let go of outside it: the drop
    // below takes that same lock, to make sure of the name it is deleting.
    let held = {
        let mut profiles = PROFILES.lock().unwrap_or_else(|hold| hold.into_inner());

        profiles.remove(&name)
    };

    if let Some(container) = held {
        // Which is the whole of it where nothing else is inside: the last hold
        // going is [`Container::drop`], and that is where a container ends. A
        // session still running holds one too, and then this is the *second*
        // last hold and the ending is that session's — see the drop.
        drop(container);

        return;
    }

    // Nothing held under that name, so what there is to go on is what the
    // server that made it wrote down. Nothing at all where there is no record:
    // a Conversation whose sessions never ran on this platform has no profile
    // to delete and no entry anywhere naming it.
    let Some(remembered) = remembering::read(data_dir, conversation) else {
        return;
    };

    tracing::info!(
        name = remembered.name,
        conversation,
        entries = remembered.entries.len(),
        "an AppContainer left behind by a server that has gone is being taken off the machine \
         with the entries it was granted"
    );

    given_back(
        &remembered.name,
        &remembered.sid,
        &remembered.entries,
        Some((data_dir, conversation)),
    );
}

/// Let go of the hold on a Conversation's container without taking anything
/// back — which is what a server that died did, and what a suite makes a crash
/// out of.
///
/// **The one thing here that leaves a boundary standing on purpose.** Everything
/// else takes the entries off the human's directories and deletes the profile;
/// this leaves both exactly where they are, with the record still under the Data
/// Directory, which is the state the next server's sweep has to be able to
/// clear. Nothing in the server calls it — see
/// `crates/server/tests/sandbox_windows.rs`, which is the whole of why it is
/// here.
pub fn forgotten(data_dir: &Path, conversation: i64) {
    let name = profile(data_dir, conversation);

    let held = {
        let mut profiles = PROFILES.lock().unwrap_or_else(|hold| hold.into_inner());

        profiles.remove(&name)
    };

    // Leaked rather than dropped, because dropping is precisely what a process
    // that died did not do: what is being made here is a machine carrying a
    // profile and its entries with nothing holding either.
    if let Some(container) = held {
        std::mem::forget(container);
    }
}

/// What the profile of `conversation`'s container is called on this machine.
///
/// Both halves because both are needed — see [`Container::for_conversation`],
/// which is where the whole of that is.
///
/// Public for the reason the module is: what says a profile has really gone is
/// making one under its name and being allowed to, and the suite is what asks —
/// see `crates/server/tests/sandbox_windows.rs`.
pub fn profile(data_dir: &Path, conversation: i64) -> String {
    format!("{}-{conversation}", crate::pipe::bare(data_dir))
}

/// The container of `conversation`, made where nothing is holding one and
/// shared where something is.
///
/// **A profile of this name that Verkstead is not holding is one a run before
/// this left behind**, and it is taken away rather than worked around. The name
/// is the Data Directory's fingerprint and a Conversation's id, so nothing else
/// on the machine writes it: what is under it is a container of Verkstead's own
/// that a crash got in the way of deleting, and a Conversation whose sessions
/// were refused for ever afterwards would be a worse answer than a fresh
/// profile. The SID is derived from the name, so what comes back is the
/// identity the entries left on those directories already name.
///
/// **And it is written down before it is handed to anybody.** A container
/// nothing wrote down is one no later server could take away — see this
/// module's own documentation — so a record that will not write refuses the
/// container, and the profile made a moment ago goes with the refusal rather
/// than being left for the next sweep to puzzle over.
///
/// The map is held for the whole of this, which is what makes two callers
/// arriving at once one profile rather than two.
fn held(data_dir: &Path, conversation: i64) -> io::Result<Arc<Container>> {
    let name = profile(data_dir, conversation);

    let mut profiles = PROFILES.lock().unwrap_or_else(|held| held.into_inner());

    if let Some(container) = profiles.get(&name) {
        return Ok(container.clone());
    }

    let kept = Kept {
        data_dir: data_dir.to_owned(),
        conversation,
    };

    let made = Container::made(&name, Some(kept.clone())).or_else(|refused| {
        tracing::warn!(
            name,
            error = ?refused,
            "an AppContainer profile of this Conversation's own name is on the machine and \
             nothing here is holding it, so it is one a run before this one left behind: it \
             was deleted and made again"
        );

        unsafe { DeleteAppContainerProfile(wide(OsStr::new(&name)).as_ptr()) };

        Container::made(&name, Some(kept))
    })?;

    let container = Arc::new(made);
    profiles.insert(name, container.clone());

    Ok(container)
}

/// The container process `id` is running inside, as the SID naming one — and
/// nothing where it is inside none.
///
/// **Asked of the token rather than assumed from how the process was started.**
/// A `CreateProcessW` that took the attribute list and started the process
/// outside a container anyway would look exactly like one that worked, so what
/// says a session is inside its container is the operating system's own answer
/// about the token it gave it. The suite is what asks.
pub fn around(id: u32) -> io::Result<Option<String>> {
    let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, id) };

    if process.is_null() {
        return Err(io::Error::last_os_error());
    }

    let process = Handle(process);
    let mut token: HANDLE = ptr::null_mut();

    if unsafe { OpenProcessToken(process.0, TOKEN_QUERY, &mut token) } == 0 {
        return Err(io::Error::last_os_error());
    }

    let token = Handle(token);
    let mut read = 0u32;

    // The structure is one pointer and the SID it points at is written after
    // it, so the buffer is that structure and room for a SID behind it. A
    // number rather than the two calls this is usually asked with, because
    // there is a largest SID there can be — Win32's own `SECURITY_MAX_SID_SIZE`
    // is 68 bytes — and this is comfortably past it. A token inside no
    // container at all answers with the pointer left empty.
    let mut buffer = vec![0u8; size_of::<TOKEN_APPCONTAINER_INFORMATION>() + 256];

    let asked = unsafe {
        GetTokenInformation(
            token.0,
            TokenAppContainerSid,
            buffer.as_mut_ptr().cast(),
            u32::try_from(buffer.len()).unwrap_or(u32::MAX),
            &mut read,
        )
    };

    if asked == 0 {
        return Err(io::Error::last_os_error());
    }

    // Safety: the call above wrote a `TOKEN_APPCONTAINER_INFORMATION` at the
    // top of the buffer, which is what it was asked for.
    let inside = unsafe {
        buffer
            .as_ptr()
            .cast::<TOKEN_APPCONTAINER_INFORMATION>()
            .read_unaligned()
            .TokenAppContainer
    };

    // Which is what a token outside every container answers with: the structure
    // is there and the SID it points at is nothing.
    if inside.is_null() {
        return Ok(None);
    }

    Ok(written(inside))
}

/// What a call that would not do it answered, in the spelling an HRESULT is
/// read in.
///
/// Decimal is what a status code is least legible as, and the two answers this
/// call really gives are worth naming outright: a name already taken, which is
/// the ordinary refusal, and a machine with no AppContainers at all, which is
/// what a Verkstead running under an emulation of Windows is told.
fn said(made: i32) -> String {
    let hresult = format!("{made:#010x}");

    // A Win32 error carried as an HRESULT: `0x8007` is what says so, and the
    // low half is the number this machine has a sentence for — the sentence for
    // a profile that is already there among them.
    if made as u32 & 0xffff_0000 == 0x8007_0000 {
        return format!(
            "{hresult} ({})",
            io::Error::from_raw_os_error(made & 0xffff)
        );
    }

    match made as u32 {
        0x8000_4001 => format!("{hresult} (E_NOTIMPL: this machine has no AppContainers)"),
        _ => hresult,
    }
}

/// A SID as a person reads one, or nothing where it would not be written down.
fn written(sid: PSID) -> Option<String> {
    let mut put: *mut u16 = ptr::null_mut();

    if unsafe { ConvertSidToStringSidW(sid, &mut put) } == 0 || put.is_null() {
        return None;
    }

    let mut length = 0;

    // Safety: what the call wrote is a string ending in a nothing, and this is
    // how long it is.
    while unsafe { *put.add(length) } != 0 {
        length += 1;
    }

    // Safety: `length` units are what was written, and they are read before the
    // block holding them is given back.
    let text = OsString::from_wide(unsafe { std::slice::from_raw_parts(put, length) })
        .to_string_lossy()
        .into_owned();

    unsafe { LocalFree(put.cast()) };

    Some(text)
}

/// A SID read out of the spelling a person writes, freed when it is let go of.
///
/// Two things want one: the capability a profile is created with, and the
/// identity an access-control entry is written for — see
/// [`super::granting::writing`], which is the other. A second reading of the
/// same spelling would be a second answer to *which identity is this*.
pub(crate) struct Sid(PSID);

impl Sid {
    /// One read out of the way it is written down, or a refusal saying which
    /// spelling would not resolve.
    pub(crate) fn of(sid: &str) -> io::Result<Sid> {
        let mut read: PSID = ptr::null_mut();

        if unsafe { ConvertStringSidToSidW(wide(OsStr::new(sid)).as_ptr(), &mut read) } == 0 {
            return Err(io::Error::other(format!(
                "the identity {sid} would not resolve: {}",
                io::Error::last_os_error()
            )));
        }

        Ok(Sid(read))
    }

    /// What a call that wants one is handed.
    pub(crate) fn as_psid(&self) -> PSID {
        self.0
    }

    /// How many bytes of it there are, which is what a list built around one
    /// has to be big enough for.
    pub(crate) fn length(&self) -> usize {
        usize::try_from(unsafe { GetLengthSid(self.0) }).unwrap_or(0)
    }

    /// And whether the bytes at `theirs` are this identity — which is how an
    /// entry already on a directory is told from somebody else's.
    ///
    /// The bytes rather than the spelling, because that is what a list holds:
    /// an entry carries a SID after its mask and never the text of one.
    pub(crate) fn is(&self, theirs: &[u8]) -> bool {
        theirs.len() >= self.length()
            && unsafe { EqualSid(self.0, theirs.as_ptr().cast_mut().cast()) } != 0
    }
}

impl Drop for Sid {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { LocalFree(self.0.cast()) };
        }
    }
}

/// And one Windows itself made, which is given back its own way.
struct Made(PSID);

impl Drop for Made {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { FreeSid(self.0) };
        }
    }
}
