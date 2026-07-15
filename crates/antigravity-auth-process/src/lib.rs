//! Safe official-client discovery and bounded argv-based process execution.

use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

/// Supported official client executable families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OfficialClient {
    /// Google Gemini CLI.
    GeminiCli,
    /// Antigravity CLI.
    AntigravityCli,
}

impl OfficialClient {
    /// Return the default executable filename for this platform.
    #[must_use]
    pub const fn executable_name(self) -> &'static str {
        match self {
            Self::GeminiCli => "gemini",
            Self::AntigravityCli => "agy",
        }
    }
}

/// Validated path to an official client executable.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveredClient {
    kind: OfficialClient,
    executable: PathBuf,
}

impl DiscoveredClient {
    /// Return the discovered client family.
    #[must_use]
    pub const fn kind(&self) -> OfficialClient {
        self.kind
    }

    /// Return the validated executable path.
    #[must_use]
    pub fn executable(&self) -> &Path {
        &self.executable
    }

    /// Run the allowlisted `--version` probe with a small output budget.
    ///
    /// # Errors
    ///
    /// Returns a process error when spawning, waiting, reading, or the probe status fails.
    pub fn probe_version(&self, timeout: Duration) -> Result<VersionProbe, ProcessError> {
        let output = run_bounded(&self.executable, [OsStr::new("--version")], timeout, 4096)?;
        if !output.status.success() {
            return Err(ProcessError::ProbeFailed(output.status.code()));
        }
        let stdout = String::from_utf8_lossy(&output.stdout);
        let version = stdout
            .lines()
            .map(str::trim)
            .find(|line| !line.is_empty())
            .ok_or(ProcessError::EmptyVersion)?;
        if version.chars().any(char::is_control) || version.len() > 256 {
            return Err(ProcessError::InvalidVersion);
        }
        Ok(VersionProbe {
            version: version.to_owned(),
            truncated: output.stdout_truncated || output.stderr_truncated,
        })
    }
}

/// Safe, bounded version-probe result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionProbe {
    version: String,
    truncated: bool,
}

impl VersionProbe {
    /// Return the first non-empty version line.
    #[must_use]
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Report whether either process stream exceeded the capture budget.
    #[must_use]
    pub const fn truncated(&self) -> bool {
        self.truncated
    }
}

/// Discover an official client through an explicit override or a supplied PATH value.
///
/// Supplying PATH explicitly keeps discovery deterministic and prevents hidden environment capture.
///
/// # Errors
///
/// Returns a typed error when an override is unsafe or no executable is found.
pub fn discover_client(
    kind: OfficialClient,
    explicit: Option<&Path>,
    search_path: Option<&OsStr>,
) -> Result<DiscoveredClient, DiscoveryError> {
    if let Some(path) = explicit {
        validate_executable(path)?;
        return Ok(DiscoveredClient {
            kind,
            executable: path.to_owned(),
        });
    }

    let search_path = search_path.ok_or(DiscoveryError::MissingSearchPath)?;
    for directory in std::env::split_paths(search_path) {
        for candidate in executable_candidates(&directory, kind.executable_name()) {
            if validate_executable(&candidate).is_ok() {
                return Ok(DiscoveredClient {
                    kind,
                    executable: candidate,
                });
            }
        }
    }
    Err(DiscoveryError::NotFound(kind.executable_name()))
}

fn executable_candidates(directory: &Path, name: &str) -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        vec![directory.join(format!("{name}.exe")), directory.join(name)]
    }
    #[cfg(not(windows))]
    {
        vec![directory.join(name)]
    }
}

fn validate_executable(path: &Path) -> Result<(), DiscoveryError> {
    if !path.is_absolute() {
        return Err(DiscoveryError::RelativeOverride);
    }
    let metadata = fs::symlink_metadata(path).map_err(|error| match error.kind() {
        io::ErrorKind::NotFound => DiscoveryError::NotFound("explicit executable"),
        _ => DiscoveryError::Io(error),
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(DiscoveryError::UnsafeFileType);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(DiscoveryError::NotExecutable);
        }
    }
    Ok(())
}

/// Bounded process output. Treat bytes as sensitive unless the invoked probe is explicitly safe.
#[derive(Debug)]
pub struct ProcessOutput {
    /// Child exit status.
    pub status: ExitStatus,
    /// Bounded stdout prefix.
    pub stdout: Vec<u8>,
    /// Bounded stderr prefix.
    pub stderr: Vec<u8>,
    /// Whether stdout exceeded the capture budget.
    pub stdout_truncated: bool,
    /// Whether stderr exceeded the capture budget.
    pub stderr_truncated: bool,
}

/// Run an executable directly with argv values, bounded output, and a wall-clock timeout.
///
/// The child inherits no stdin. Output readers continue draining after their capture budget is
/// reached, preventing pipe deadlock while retaining only a bounded prefix.
///
/// # Errors
///
/// Returns an error when spawning, waiting, killing, or reading process streams fails, or on timeout.
pub fn run_bounded<I, S>(
    executable: &Path,
    arguments: I,
    timeout: Duration,
    max_stream_bytes: usize,
) -> Result<ProcessOutput, ProcessError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if timeout.is_zero() || max_stream_bytes == 0 {
        return Err(ProcessError::InvalidLimit);
    }
    let mut child = Command::new(executable)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let stdout = child.stdout.take().ok_or(ProcessError::MissingPipe)?;
    let stderr = child.stderr.take().ok_or(ProcessError::MissingPipe)?;
    let stdout_reader = thread::spawn(move || drain_bounded(stdout, max_stream_bytes));
    let stderr_reader = thread::spawn(move || drain_bounded(stderr, max_stream_bytes));
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill()?;
            let _ = child.wait();
            join_reader(stdout_reader)?;
            join_reader(stderr_reader)?;
            return Err(ProcessError::TimedOut);
        }
        thread::sleep(Duration::from_millis(10));
    };
    let (stdout, stdout_truncated) = join_reader(stdout_reader)?;
    let (stderr, stderr_truncated) = join_reader(stderr_reader)?;
    Ok(ProcessOutput {
        status,
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
    })
}

fn drain_bounded(mut reader: impl Read, limit: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut captured = Vec::with_capacity(limit.min(8192));
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(captured.len());
        let retained = remaining.min(count);
        captured.extend_from_slice(&buffer[..retained]);
        truncated |= retained < count;
    }
    Ok((captured, truncated))
}

fn join_reader(
    handle: thread::JoinHandle<io::Result<(Vec<u8>, bool)>>,
) -> Result<(Vec<u8>, bool), ProcessError> {
    handle
        .join()
        .map_err(|_| ProcessError::ReaderPanicked)?
        .map_err(Into::into)
}

/// Safe executable-discovery failure.
#[derive(Debug)]
pub enum DiscoveryError {
    /// No deterministic PATH value was supplied.
    MissingSearchPath,
    /// No matching executable was found.
    NotFound(&'static str),
    /// Explicit overrides must be absolute.
    RelativeOverride,
    /// Path is a symlink or not a regular file.
    UnsafeFileType,
    /// File lacks executable permission on this platform.
    NotExecutable,
    /// Filesystem metadata could not be read.
    Io(io::Error),
}

impl std::fmt::Display for DiscoveryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSearchPath => formatter.write_str("client search PATH was not supplied"),
            Self::NotFound(name) => {
                write!(formatter, "official client executable not found: {name}")
            }
            Self::RelativeOverride => formatter.write_str("client override must be absolute"),
            Self::UnsafeFileType => {
                formatter.write_str("client executable is not a safe regular file")
            }
            Self::NotExecutable => formatter.write_str("client file is not executable"),
            Self::Io(_) => formatter.write_str("client executable metadata could not be read"),
        }
    }
}

impl std::error::Error for DiscoveryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

/// Bounded child-process failure.
#[derive(Debug)]
pub enum ProcessError {
    /// Limits must be positive.
    InvalidLimit,
    /// Child pipe was unexpectedly unavailable.
    MissingPipe,
    /// Child exceeded the wall-clock limit and was terminated.
    TimedOut,
    /// Output-reader thread panicked.
    ReaderPanicked,
    /// Version probe returned a non-success status.
    ProbeFailed(Option<i32>),
    /// Version probe returned no non-empty stdout line.
    EmptyVersion,
    /// Version line was oversized or contained control characters.
    InvalidVersion,
    /// Process or pipe I/O failed.
    Io(io::Error),
}

impl From<io::Error> for ProcessError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidLimit => formatter.write_str("process limits must be positive"),
            Self::MissingPipe => formatter.write_str("child output pipe is unavailable"),
            Self::TimedOut => formatter.write_str("official client probe timed out"),
            Self::ReaderPanicked => formatter.write_str("child output reader failed"),
            Self::ProbeFailed(code) => write!(formatter, "official client probe failed: {code:?}"),
            Self::EmptyVersion => formatter.write_str("official client returned no version"),
            Self::InvalidVersion => {
                formatter.write_str("official client returned an invalid version")
            }
            Self::Io(_) => formatter.write_str("official client process I/O failed"),
        }
    }
}

impl std::error::Error for ProcessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::{DiscoveryError, OfficialClient, ProcessError, discover_client, run_bounded};
    use std::ffi::{OsStr, OsString};
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn fixture(name: &str, body: &str) -> (PathBuf, PathBuf) {
        let directory = std::env::temp_dir().join(format!(
            "antigravity-auth-process-{}-{}-{}-{}",
            std::process::id(),
            name,
            FIXTURE_SEQUENCE.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock after Unix epoch")
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&directory);
        fs::create_dir(&directory).expect("create fixture directory");
        let executable = directory.join(name);
        fs::write(&executable, format!("#!/bin/sh\n{body}\n")).expect("write fixture");
        let mut permissions = fs::metadata(&executable).expect("metadata").permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions).expect("make executable");
        (directory, executable)
    }

    fn remove(directory: &Path) {
        fs::remove_dir_all(directory).expect("remove fixture directory");
    }

    #[test]
    fn explicit_version_probe_is_bounded() {
        let (directory, executable) = fixture("agy", "printf '1.1.2\\n'");
        let client = discover_client(OfficialClient::AntigravityCli, Some(&executable), None)
            .expect("discover explicit client");
        let version = client
            .probe_version(Duration::from_secs(1))
            .expect("probe version");
        assert_eq!(version.version(), "1.1.2");
        assert!(!version.truncated());
        remove(&directory);
    }

    #[test]
    fn path_search_is_deterministic() {
        let (directory, executable) = fixture("gemini", "printf 'test\\n'");
        let path = std::env::join_paths([&directory]).expect("join PATH");
        let client = discover_client(OfficialClient::GeminiCli, None, Some(&path))
            .expect("discover PATH client");
        assert_eq!(client.executable(), executable);
        remove(&directory);
    }

    #[test]
    fn argv_metacharacters_are_not_evaluated() {
        let (directory, executable) = fixture("argv", "printf '%s' \"$1\"");
        let marker = directory.join("must-not-exist");
        let argument = OsString::from(format!("value; touch {}", marker.display()));
        let output = run_bounded(&executable, [&argument], Duration::from_secs(1), 1024)
            .expect("run fixture");
        assert_eq!(output.stdout, argument.as_encoded_bytes());
        assert!(!marker.exists());
        remove(&directory);
    }

    #[test]
    fn output_is_truncated_without_deadlock() {
        let (directory, executable) = fixture("large", "yes x | head -c 100000");
        let output = run_bounded(
            &executable,
            std::iter::empty::<&OsStr>(),
            Duration::from_secs(2),
            128,
        )
        .expect("run large-output fixture");
        assert_eq!(output.stdout.len(), 128);
        assert!(output.stdout_truncated);
        remove(&directory);
    }

    #[test]
    fn timeout_terminates_child() {
        let (directory, executable) = fixture("slow", "exec sleep 2");
        let result = run_bounded(
            &executable,
            std::iter::empty::<&OsStr>(),
            Duration::from_millis(50),
            128,
        );
        assert!(matches!(result, Err(ProcessError::TimedOut)));
        remove(&directory);
    }

    #[test]
    fn unsafe_overrides_fail_closed() {
        let directory = std::env::temp_dir();
        assert!(matches!(
            discover_client(OfficialClient::GeminiCli, Some(Path::new("relative")), None),
            Err(DiscoveryError::RelativeOverride)
        ));
        assert!(matches!(
            discover_client(OfficialClient::GeminiCli, Some(&directory), None),
            Err(DiscoveryError::UnsafeFileType)
        ));
    }
}
