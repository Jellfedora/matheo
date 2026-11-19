//! Updates from floor (the releases Mathéo's CI sends there, see floor's src/releases.js), as Ronnie
//! does: a check at the start, every hour and when asked (his menu) finds a newer release; Mathéo says so, and once
//! accepted (a click on his bubble, or his menu), the archive built for this system is downloaded,
//! checked against its SHA-256 digest and its Ed25519 signature (made by the CI with a key floor never
//! sees), swapped in place of the running app, which then starts again.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const BASE: &str = "https://185.245.143.130/floor";
/// The key the release archives are signed with (Ed25519, raw, base64): Ronnie's release key, its
/// private half a secret of the CI (RONNIE_UPDATE_KEY), kept in ~/.ronnie-signing/update-ed25519.pem.
const UPDATE_KEY: &str = "nH6V6OBML6aG9YXBSyUH8hxvaWRzK0CjgvF0TA+f/jw=";
/// Refuse absurd downloads (the archive is a few MB).
const MAX_DOWNLOAD: u64 = 100 * 1024 * 1024;

/// This system's archive (names made by scripts/package.sh).
const ARCHIVE: &str = if cfg!(target_os = "macos") {
    "matheo-universal-apple-darwin.tar.gz"
} else if cfg!(windows) {
    "matheo-x86_64-pc-windows-msvc.zip"
} else {
    "matheo-x86_64-unknown-linux-gnu.tar.gz"
};

/// floor's latest release (GET /v1/matheo/release).
#[derive(Deserialize)]
struct Release {
    version: String,
    assets: Vec<Asset>,
}

#[derive(Clone, Deserialize)]
struct Asset {
    name: String,
    /// Its path on floor.
    file: String,
    sha256: String,
    /// Ed25519, base64.
    signature: String,
}

#[derive(Clone, Default, PartialEq, Debug)]
pub(crate) enum State {
    #[default]
    Idle,
    /// Looking for a newer version, asked from his menu.
    Checking,
    /// Asked: this version is the latest.
    Latest,
    /// Asked: floor couldn't be reached.
    Unreachable,
    /// A newer version, offered.
    Available(String),
    Installing(String),
    /// Installed: he starts again.
    Installed(String),
    Failed(String),
}

static STATE: Mutex<State> = Mutex::new(State::Idle);
static FOUND: Mutex<Option<(String, Asset)>> = Mutex::new(None);

pub(crate) fn state() -> State {
    STATE.lock().map(|s| s.clone()).unwrap_or_default()
}

fn set(state: State) {
    if let Ok(mut s) = STATE.lock() {
        *s = state;
    }
}

fn agent(total: Duration) -> ureq::Agent {
    ureq::config::Config::builder().timeout_connect(Some(Duration::from_secs(15))).timeout_global(Some(total)).build().new_agent()
}

/// (major, minor, patch).
fn parse(v: &str) -> Option<(u64, u64, u64)> {
    let core = v.trim_start_matches('v').split(['-', '+']).next()?;
    let mut parts = core.split('.').map(str::parse::<u64>);
    Some((parts.next()?.ok()?, parts.next()?.ok()?, parts.next()?.ok()?))
}

pub(crate) fn is_newer(candidate: &str, current: &str) -> bool {
    matches!((parse(candidate), parse(current)), (Some(a), Some(b)) if a > b)
}

fn latest() -> Result<Option<(String, Asset)>> {
    let release: Release = agent(Duration::from_secs(30))
        .get(format!("{BASE}/v1/matheo/release"))
        .header("User-Agent", concat!("matheo/", env!("CARGO_PKG_VERSION")))
        .call()
        .context("floor unreachable")?
        .body_mut()
        .read_json()?;
    let version = release.version.trim_start_matches('v').to_owned();
    if !is_newer(&version, VERSION) {
        return Ok(None);
    }
    let asset = release.assets.into_iter().find(|a| a.name == ARCHIVE).with_context(|| format!("version {version} has no {ARCHIVE}"))?;
    Ok(Some((version, asset)))
}

/// Nothing being installed: a check may start.
fn free() -> bool {
    matches!(state(), State::Idle | State::Latest | State::Unreachable | State::Available(_) | State::Failed(_))
}

/// A newer release found: kept, offered.
fn found(version: String, asset: Asset) {
    if let Ok(mut found) = FOUND.lock() {
        *found = Some((version.clone(), asset));
    }
    set(State::Available(version));
}

/// Looks for a newer release at the start, then every hour.
pub(crate) fn watch() {
    std::thread::spawn(|| {
        loop {
            if free()
                && let Ok(Some((version, asset))) = latest()
            {
                found(version, asset);
            }
            std::thread::sleep(Duration::from_secs(3600));
        }
    });
}

/// Asked from his menu: looks now, and says what it found (`Available`, `Latest`, `Unreachable`).
pub(crate) fn check() {
    if !free() {
        return;
    }
    set(State::Checking);
    std::thread::spawn(|| match latest() {
        Ok(Some((version, asset))) => found(version, asset),
        Ok(None) => set(State::Latest),
        Err(_) => set(State::Unreachable),
    });
}

/// Accepted: downloads and installs the version offered in the background, then starts again.
pub(crate) fn install() {
    let Some((version, asset)) = FOUND.lock().ok().and_then(|f| f.clone()) else { return };
    if !matches!(state(), State::Available(_) | State::Failed(_)) {
        return;
    }
    set(State::Installing(version.clone()));
    std::thread::spawn(move || {
        match std::env::current_exe().context("unknown executable").and_then(|exe| install_from(&exe, &asset)) {
            Ok(()) => set(State::Installed(version)),
            Err(e) => set(State::Failed(format!("{e:#}"))),
        }
    });
}

/// Whether `bytes` carry this signature (base64) by the release key.
fn signed(bytes: &[u8], signature: &str) -> bool {
    use base64::Engine as _;
    let engine = base64::engine::general_purpose::STANDARD;
    let (Ok(key), Ok(signature)) = (engine.decode(UPDATE_KEY), engine.decode(signature.trim())) else { return false };
    ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519, key).verify(bytes, &signature).is_ok()
}

fn download(asset: &Asset) -> Result<Vec<u8>> {
    use std::io::Read as _;
    let url = format!("{BASE}/{}", asset.file.trim_start_matches('/').trim_start_matches("floor/"));
    let mut response = agent(Duration::from_secs(600)).get(&url).header("User-Agent", concat!("matheo/", env!("CARGO_PKG_VERSION"))).call().context("download failed")?;
    let mut bytes = Vec::new();
    response.body_mut().with_config().limit(MAX_DOWNLOAD).reader().read_to_end(&mut bytes).context("download failed")?;
    let actual: String = Sha256::digest(&bytes).iter().map(|b| format!("{b:02x}")).collect();
    if !actual.eq_ignore_ascii_case(&asset.sha256) {
        bail!("the downloaded archive is corrupted (SHA-256 mismatch)");
    }
    Ok(bytes)
}

fn install_from(exe: &Path, asset: &Asset) -> Result<()> {
    let archive = download(asset)?;
    // Signed by the CI, or not installed (whoever serves the archive).
    if !signed(&archive, &asset.signature) {
        bail!("the archive's signature is not ours: not installed");
    }
    #[cfg(target_os = "macos")]
    if let Some(bundle) = app_bundle(exe) {
        return replace_bundle(&bundle, &archive);
    }
    let staging = tempdir(exe.parent().context("unknown executable directory")?)?;
    let result = (|| {
        extract(&archive, &staging)?;
        let name = if cfg!(windows) { "matheo.exe" } else { "matheo" };
        let new = find(&staging, name).with_context(|| format!("{name} missing from the archive"))?;
        #[cfg(not(target_os = "macos"))]
        return self_replace::self_replace(&new).context("could not replace the executable");
        #[cfg(target_os = "macos")]
        {
            std::fs::rename(&new, exe).context("could not replace the executable")
        }
    })();
    let _ = std::fs::remove_dir_all(&staging);
    result
}

/// A fresh directory next to `near` (same filesystem, so moves are renames).
fn tempdir(near: &Path) -> Result<PathBuf> {
    let dir = near.join(format!(".matheo-update-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir(&dir).with_context(|| format!("cannot write in {}", near.display()))?;
    Ok(dir)
}

fn extract(archive: &[u8], into: &Path) -> Result<()> {
    #[cfg(windows)]
    zip::ZipArchive::new(std::io::Cursor::new(archive))?.extract(into)?;
    #[cfg(not(windows))]
    tar::Archive::new(flate2::read::GzDecoder::new(archive)).unpack(into)?;
    Ok(())
}

/// First file or directory called `name` under `dir`.
fn find(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if entry.file_name() == name {
            return Some(path);
        }
        if path.is_dir()
            && let Some(found) = find(&path, name)
        {
            return Some(found);
        }
    }
    None
}

/// The `.app` bundle containing `exe`, when running from one (where the user has it, not macOS'
/// read-only copy of a downloaded app).
#[cfg(target_os = "macos")]
fn app_bundle(exe: &Path) -> Option<PathBuf> {
    let bundle = exe.ancestors().find(|p| p.extension().is_some_and(|e| e == "app")).map(Path::to_path_buf)?;
    if !bundle.components().any(|c| c.as_os_str() == "AppTranslocation") {
        return Some(bundle);
    }
    Some(untranslocated(&bundle).unwrap_or(bundle))
}

/// Where a translocated bundle really is (Security framework, as Sparkle asks it).
#[cfg(target_os = "macos")]
fn untranslocated(bundle: &Path) -> Option<PathBuf> {
    use objc2::rc::Retained;
    use objc2_foundation::{NSString, NSURL};
    #[link(name = "Security", kind = "framework")]
    unsafe extern "C" {
        fn SecTranslocateCreateOriginalPathForURL(translocated: *const NSURL, error: *mut *const std::ffi::c_void) -> *mut NSURL;
    }
    let url = NSURL::fileURLWithPath(&NSString::from_str(&bundle.to_string_lossy()));
    // SAFETY: CFURL and NSURL are toll-free bridged; the result is owned (+1), released by `Retained`.
    let original = unsafe { Retained::from_raw(SecTranslocateCreateOriginalPathForURL(&*url, std::ptr::null_mut()))? };
    let path = PathBuf::from(original.path()?.to_string());
    (path != bundle).then_some(path)
}

/// Swaps the whole bundle, rolling back if the new one can't be moved in.
#[cfg(target_os = "macos")]
fn replace_bundle(bundle: &Path, archive: &[u8]) -> Result<()> {
    let parent = bundle.parent().context("unknown application directory")?;
    let staging = tempdir(parent)?;
    let result = (|| {
        extract(archive, &staging)?;
        let new = find(&staging, "Mathéo.app").or_else(|| find(&staging, "Matheo.app")).context("Mathéo.app missing from the archive")?;
        let old = staging.join("previous.app");
        std::fs::rename(bundle, &old).with_context(|| format!("cannot replace {}", bundle.display()))?;
        if let Err(e) = std::fs::rename(&new, bundle) {
            let _ = std::fs::rename(&old, bundle);
            return Err(e).context("could not install the new version");
        }
        Ok(())
    })();
    let _ = std::fs::remove_dir_all(&staging);
    result
}

/// Starts the updated Mathéo once this one has quit (it holds the one-instance port until then); the
/// caller then quits.
pub(crate) fn relaunch() {
    let Ok(exe) = std::env::current_exe() else { return };
    #[cfg(target_os = "macos")]
    let (program, args): (PathBuf, Vec<std::ffi::OsString>) = match app_bundle(&exe) {
        Some(bundle) => ("/usr/bin/open".into(), vec!["-n".into(), bundle.into()]),
        None => (exe, Vec::new()),
    };
    #[cfg(not(target_os = "macos"))]
    let (program, args): (PathBuf, Vec<std::ffi::OsString>) = (exe, Vec::new());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        const SCRIPT: &str = r#"pid=$1; shift; i=0
while kill -0 "$pid" 2>/dev/null && [ "$i" -lt 100 ]; do sleep 0.1; i=$((i + 1)); done
exec "$@""#;
        let _ = Command::new("/bin/sh").args(["-c", SCRIPT, "sh", &std::process::id().to_string()]).arg(&program).args(&args).process_group(0).spawn();
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // Waits a moment (this one gone, its port free), then starts the new one, without a console.
        let _ = Command::new("cmd").args(["/C", "timeout", "/T", "2", "/NOBREAK", ">NUL", "&", "start", ""]).arg(&program).args(&args).creation_flags(0x0800_0000).spawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compares_versions() {
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(is_newer("1.0.0", "0.10.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("garbage", "0.1.0"));
    }

    /// A signature made by the release key (openssl pkeyutl -sign -rawin, as the CI does).
    #[test]
    fn checks_the_signature() {
        let sig = "ZqJjpCCzVHq6qH9cAzP6P1PsG6Zu5xLwEkEk1Wh0raoqP+RJQ/cnDX2etEfZeIBaefpT5WH80LKFgdzZgluEDg==";
        assert!(signed(b"ronnie update test", sig));
        assert!(!signed(b"ronnie update tesT", sig));
    }

    #[test]
    fn finds_nested_file() {
        let dir = std::env::temp_dir().join(format!("matheo-find-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("a/b")).unwrap();
        std::fs::write(dir.join("a/b/matheo"), b"x").unwrap();
        assert_eq!(find(&dir, "matheo"), Some(dir.join("a/b/matheo")));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
