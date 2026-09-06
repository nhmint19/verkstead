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
//! **What creates one is what deletes it, and the entries go with it.** A
//! [`Container`] takes back every access-control entry written for its identity
//! and then deletes its profile as it is dropped, so a boundary proved by
//! attempting it leaves the machine as it found it. The order is that way round
//! deliberately: the SID is what names an entry, so a profile deleted first
//! would leave entries standing on the human's own directories as a number
//! nothing can resolve.
//!
//! **One is held by everything running inside it.** A Conversation can have a
//! session and a Conversation Terminal going at once and both are inside the
//! same profile, so [`Container::for_conversation`] hands out a shared one and
//! the profile goes when the last of them does — see [`held`]. Giving it the
//! Conversation's own lifetime instead — granted at the first session, deleted
//! with the Worktree, swept at startup — is the task after this one's.

use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::io;
use std::os::windows::ffi::OsStringExt;
use std::path::Path;
use std::ptr;
use std::sync::{Arc, LazyLock, Mutex, Weak};

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

use super::granting::{Entry, writing};
use super::starting::{Handle, wide};

/// Every profile this process is holding, by the name it was made under.
///
/// **A profile is one machine-wide thing and this is what keeps it one.** Two
/// things run inside a Conversation's container — a grilling session and the
/// Conversation Terminal beside it — and each of them asks for the container as
/// it starts. Made twice, the second would be refused the name the first is
/// holding; deleted twice, the first to end would take the profile out from
/// under the one still running. So what is handed out is shared, and what is
/// kept here is a weak hold that says nothing about how long it lives.
static PROFILES: LazyLock<Mutex<HashMap<String, Weak<Container>>>> =
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
}

impl Container {
    /// The container a session of `conversation` runs in, under the Data
    /// Directory at `data_dir` — shared with everything else already running
    /// inside it.
    ///
    /// The name is both halves because both are needed: the Data Directory so
    /// that two Verksteads on one machine are two sets of containers, and the
    /// Conversation so that what one Conversation's session may reach is not
    /// what the next one's may — see this module's own documentation.
    pub fn for_conversation(data_dir: &Path, conversation: i64) -> io::Result<Arc<Container>> {
        held(&format!("{}-{conversation}", crate::pipe::bare(data_dir)))
    }

    /// The entries written for this identity, remembered so that they can be
    /// taken back when it goes.
    ///
    /// Added to rather than replaced: a Conversation's second session describes
    /// its own surface, and an entry written by the first is still an entry on
    /// somebody's directory.
    pub(crate) fn wrote(&self, entries: Vec<Entry>) {
        let mut granted = self.granted.lock().unwrap_or_else(|held| held.into_inner());

        for entry in entries {
            if !granted.contains(&entry) {
                granted.push(entry);
            }
        }
    }

    /// And one under a name said outright, which is what the suite makes.
    ///
    /// **A name already taken is an error rather than the profile that is
    /// there.** A container Verkstead did not make is one whose capabilities
    /// and whose grants are somebody else's, and a session started inside it
    /// would be a session behind a boundary nothing here described (ADR-0014,
    /// Q18): what cannot be made refuses the session rather than falling back.
    pub fn named(name: &str) -> io::Result<Container> {
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

        Ok(Container {
            name: name.to_owned(),
            sid,
            granted: Mutex::new(Vec::new()),
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
    /// Nothing is reported about the profile itself: one that will not delete
    /// is not something the code that let go of it can do anything about, and
    /// what would find it again is the sweep the task after this one adds.
    fn drop(&mut self) {
        let granted =
            std::mem::take(&mut *self.granted.lock().unwrap_or_else(|held| held.into_inner()));

        let mut profiles = PROFILES.lock().unwrap_or_else(|held| held.into_inner());

        // A live hold under this name is a container made *after* this one, in
        // the moment between this one's last holder letting go and this
        // running — see [`held`], which replaces a profile nothing is holding.
        // Its profile is not this one's to delete, and the entries this wrote
        // are the same SID's, so they go to it rather than being taken back
        // from under it.
        if let Some(taken) = profiles.get(&self.name).and_then(Weak::upgrade) {
            taken.wrote(granted);

            return;
        }

        // The entries first and the profile after them, which is the order the
        // probe was careful about on the human's own directories: the SID is
        // what names an entry, and a profile deleted first leaves every one of
        // them standing as a number nothing on the machine can resolve.
        //
        // Both while the map is held, so that a caller arriving in the meantime
        // waits and then makes a profile of its own rather than finding this
        // one part-way out. What that costs is a session start held up by
        // another Conversation's ending, for as long as it takes to walk back
        // the trees this was granted.
        profiles.remove(&self.name);

        writing::strip(&granted, &self.sid);

        unsafe { DeleteAppContainerProfile(wide(OsStr::new(&self.name)).as_ptr()) };
    }
}

/// The container called `name`, made where nothing is holding one and shared
/// where something is.
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
/// The map is held for the whole of this, which is what makes two callers
/// arriving at once one profile rather than two.
fn held(name: &str) -> io::Result<Arc<Container>> {
    let mut profiles = PROFILES.lock().unwrap_or_else(|held| held.into_inner());

    if let Some(container) = profiles.get(name).and_then(Weak::upgrade) {
        return Ok(container);
    }

    let made = Container::named(name).or_else(|refused| {
        tracing::warn!(
            name,
            error = ?refused,
            "an AppContainer profile of this Conversation's own name is on the machine and \
             nothing here is holding it, so it is one a run before this one left behind: it \
             was deleted and made again"
        );

        unsafe { DeleteAppContainerProfile(wide(OsStr::new(name)).as_ptr()) };

        Container::named(name)
    })?;

    let container = Arc::new(made);
    profiles.insert(name.to_owned(), Arc::downgrade(&container));

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
