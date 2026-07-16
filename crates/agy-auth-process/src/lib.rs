//! Safe official-client discovery and bounded argv-based process execution.

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
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
        parse_version_output(&output)
    }

    /// Run the allowlisted `--version` probe inside a validated isolated environment.
    ///
    /// # Errors
    ///
    /// Returns a process error when spawning, waiting, reading, or the probe status fails.
    pub fn probe_version_isolated(
        &self,
        environment: &IsolatedClientEnvironment,
        timeout: Duration,
    ) -> Result<VersionProbe, ProcessError> {
        let output = run_bounded_isolated(
            &self.executable,
            [OsStr::new("--version")],
            environment,
            timeout,
            4096,
        )?;
        parse_version_output(&output)
    }
}

fn parse_version_output(output: &ProcessOutput) -> Result<VersionProbe, ProcessError> {
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

/// Outcome of an interactive enrollment process watched for one managed-home-relative file.
#[derive(Debug)]
pub enum InteractiveCompletion {
    /// The watched file appeared and the client was terminated after persisting it.
    FileCreated,
    /// The client exited before creating the watched file.
    Exited(ExitStatus),
}

/// Explicit, minimal environment for launching a client inside a managed profile home.
///
/// Every path must name an existing absolute directory. On Unix, group or other permission bits
/// are rejected. The child receives no inherited environment variables.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IsolatedClientEnvironment {
    home: PathBuf,
    runtime_directory: PathBuf,
    search_path: OsString,
    terminal: Option<String>,
    ssh_file_fallback: bool,
}

impl IsolatedClientEnvironment {
    /// Validate the two isolation roots and construct a deterministic child environment.
    ///
    /// # Errors
    ///
    /// Returns a process error when either directory is relative, missing, linked, not a
    /// directory, or accessible by group/other users on Unix, or when `PATH` is empty.
    pub fn new(
        home: PathBuf,
        runtime_directory: PathBuf,
        search_path: OsString,
    ) -> Result<Self, ProcessError> {
        validate_isolation_directory(&home)?;
        validate_isolation_directory(&runtime_directory)?;
        if home == runtime_directory {
            return Err(ProcessError::UnsafeIsolation(
                "home and runtime directory must differ",
            ));
        }
        if search_path.is_empty() {
            return Err(ProcessError::UnsafeIsolation("PATH must not be empty"));
        }
        Ok(Self {
            home,
            runtime_directory,
            search_path,
            terminal: None,
            ssh_file_fallback: false,
        })
    }

    /// Add a validated terminal type for an interactive child.
    ///
    /// # Errors
    ///
    /// Returns a process error when `TERM` is empty, oversized, or contains characters outside the
    /// conservative terminal-name allowlist.
    pub fn with_terminal(mut self, terminal: &str) -> Result<Self, ProcessError> {
        if terminal.is_empty()
            || terminal.len() > 64
            || !terminal
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"._+-".contains(&byte))
        {
            return Err(ProcessError::UnsafeIsolation("TERM is invalid"));
        }
        self.terminal = Some(terminal.to_owned());
        Ok(self)
    }

    /// Select the official client's SSH file-backed credential mode.
    ///
    /// The synthetic loopback marker contains no caller network data. It is intended only for
    /// version-scoped clients whose SSH fallback contract has been independently verified.
    #[must_use]
    pub const fn with_ssh_file_fallback(mut self) -> Self {
        self.ssh_file_fallback = true;
        self
    }

    fn apply(&self, command: &mut Command) {
        command
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_CONFIG_HOME", self.home.join(".config"))
            .env("XDG_DATA_HOME", self.home.join(".local/share"))
            .env("XDG_STATE_HOME", self.home.join(".local/state"))
            .env("XDG_CACHE_HOME", self.home.join(".cache"))
            .env("XDG_RUNTIME_DIR", &self.runtime_directory)
            .env("PATH", &self.search_path)
            .env("LANG", "C.UTF-8");
        if let Some(terminal) = &self.terminal {
            command.env("TERM", terminal);
        }
        if self.ssh_file_fallback {
            command
                .env("SSH_CONNECTION", "127.0.0.1 40000 127.0.0.1 22")
                .env("SSH_CLIENT", "127.0.0.1 40000 22");
        }
    }
}

fn validate_isolation_directory(path: &Path) -> Result<(), ProcessError> {
    if !path.is_absolute() {
        return Err(ProcessError::UnsafeIsolation(
            "isolation directory must be absolute",
        ));
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ProcessError::UnsafeIsolation(
            "isolation directory must be a real directory",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(ProcessError::UnsafeIsolation(
                "isolation directory permissions must be owner-only",
            ));
        }
    }
    Ok(())
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
    let arguments = arguments
        .into_iter()
        .map(|argument| argument.as_ref().to_owned())
        .collect::<Vec<OsString>>();
    let mut child = spawn_bounded(executable, &arguments)?;
    collect_bounded(&mut child, timeout, max_stream_bytes)
}

/// Run a client with a cleared environment and profile-specific home/runtime roots.
///
/// This primitive is intentionally non-interactive and suitable only for synthetic probes. A
/// future interactive launcher must preserve terminal behavior without broadening the environment
/// allowlist.
///
/// # Errors
///
/// Returns an error when limits are invalid or process execution fails.
pub fn run_bounded_isolated<I, S>(
    executable: &Path,
    arguments: I,
    environment: &IsolatedClientEnvironment,
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
    let arguments = arguments
        .into_iter()
        .map(|argument| argument.as_ref().to_owned())
        .collect::<Vec<OsString>>();
    let mut child = spawn_with_retry(executable, &arguments, |command| {
        environment.apply(command);
    })?;
    collect_bounded(&mut child, timeout, max_stream_bytes)
}

/// Launch an isolated client with the caller's terminal streams attached and wait for it to exit.
///
/// Arguments are passed directly without a shell. Standard input, output, and error are inherited,
/// so interactive terminal behavior belongs to the official client while its environment remains
/// cleared and allowlisted.
///
/// # Errors
///
/// Returns an error when spawning or waiting for the client fails.
pub fn run_interactive_isolated<I, S>(
    executable: &Path,
    arguments: I,
    environment: &IsolatedClientEnvironment,
) -> Result<ExitStatus, ProcessError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    const ATTEMPTS: usize = 4;
    let arguments = arguments
        .into_iter()
        .map(|argument| argument.as_ref().to_owned())
        .collect::<Vec<OsString>>();
    for attempt in 0..ATTEMPTS {
        let mut command = Command::new(executable);
        command
            .args(&arguments)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        environment.apply(&mut command);
        match command.status() {
            Ok(status) => return Ok(status),
            Err(error) if error.raw_os_error() == Some(26) && attempt + 1 < ATTEMPTS => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error.into()),
        }
    }
    unreachable!("interactive spawn loop returns on every final attempt")
}

/// Run an interactive client in its isolated home until a watched relative file appears.
///
/// This is intended for official-client delegated enrollment. The process is terminated immediately
/// after the client persists the expected credential file, before unrelated workspace UI continues.
///
/// # Errors
///
/// Returns an error for unsafe relative paths, pre-existing watched state, spawn/wait failures, or
/// timeout.
pub fn run_interactive_isolated_until_file<I, S>(
    executable: &Path,
    arguments: I,
    environment: &IsolatedClientEnvironment,
    watched_relative_path: &Path,
    timeout: Duration,
) -> Result<InteractiveCompletion, ProcessError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    if watched_relative_path.as_os_str().is_empty()
        || watched_relative_path.is_absolute()
        || watched_relative_path
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(ProcessError::UnsafeIsolation(
            "watched enrollment path must be relative",
        ));
    }
    let watched = environment.home.join(watched_relative_path);
    if fs::symlink_metadata(&watched).is_ok() {
        return Err(ProcessError::UnsafeIsolation(
            "watched enrollment path must be absent",
        ));
    }
    let arguments = arguments
        .into_iter()
        .map(|argument| argument.as_ref().to_owned())
        .collect::<Vec<OsString>>();
    let mut command = Command::new(executable);
    command
        .args(&arguments)
        .current_dir(&environment.home)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    environment.apply(&mut command);
    let mut child = command.spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        match fs::symlink_metadata(&watched) {
            Ok(metadata)
                if metadata.is_file()
                    && !metadata.file_type().is_symlink()
                    && metadata.len() > 0 =>
            {
                thread::sleep(Duration::from_millis(100));
                let stable = fs::symlink_metadata(&watched)?;
                if stable.is_file()
                    && !stable.file_type().is_symlink()
                    && stable.len() == metadata.len()
                {
                    if child.try_wait()?.is_none() {
                        child.kill()?;
                        let _ = child.wait();
                    }
                    return Ok(InteractiveCompletion::FileCreated);
                }
            }
            Ok(metadata) if metadata.is_file() && !metadata.file_type().is_symlink() => {}
            Ok(_) => {
                child.kill()?;
                let _ = child.wait();
                return Err(ProcessError::UnsafeIsolation(
                    "watched enrollment path has unsafe type",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                child.kill()?;
                let _ = child.wait();
                return Err(error.into());
            }
        }
        if let Some(status) = child.try_wait()? {
            return Ok(InteractiveCompletion::Exited(status));
        }
        if Instant::now() >= deadline {
            child.kill()?;
            let _ = child.wait();
            return Err(ProcessError::TimedOut);
        }
        thread::sleep(Duration::from_millis(50));
    }
}

fn spawn_bounded(executable: &Path, arguments: &[OsString]) -> io::Result<Child> {
    spawn_with_retry(executable, arguments, |_| {})
}

fn spawn_with_retry(
    executable: &Path,
    arguments: &[OsString],
    configure: impl Fn(&mut Command),
) -> io::Result<Child> {
    const ATTEMPTS: usize = 4;
    for attempt in 0..ATTEMPTS {
        let mut command = Command::new(executable);
        command
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        configure(&mut command);
        match command.spawn() {
            Ok(child) => return Ok(child),
            Err(error) if error.raw_os_error() == Some(26) && attempt + 1 < ATTEMPTS => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error),
        }
    }
    unreachable!("spawn loop returns on every final attempt")
}

fn collect_bounded(
    child: &mut Child,
    timeout: Duration,
    max_stream_bytes: usize,
) -> Result<ProcessOutput, ProcessError> {
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
    /// A requested isolated process environment does not meet safety invariants.
    UnsafeIsolation(&'static str),
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
            Self::UnsafeIsolation(reason) => write!(formatter, "unsafe isolation: {reason}"),
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
    use super::{
        DiscoveryError, IsolatedClientEnvironment, OfficialClient, ProcessError, discover_client,
        run_bounded, run_bounded_isolated, run_interactive_isolated,
    };
    use std::ffi::{OsStr, OsString};
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    static FIXTURE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn fixture(name: &str, body: &str) -> (PathBuf, PathBuf) {
        let directory = std::env::temp_dir().join(format!(
            "agy-auth-process-{}-{}-{}-{}",
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
        let mut file = fs::File::create(&executable).expect("create fixture");
        file.write_all(format!("#!/bin/sh\n{body}\n").as_bytes())
            .expect("write fixture");
        file.sync_all().expect("sync fixture");
        drop(file);
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

    #[test]
    fn isolated_profiles_receive_distinct_allowlisted_environments() {
        let (directory, executable) = fixture(
            "environment",
            "printf '%s|%s|%s|%s' \"$HOME\" \"$XDG_DATA_HOME\" \"$XDG_RUNTIME_DIR\" \"${USER-unset}\"",
        );
        let work_home = directory.join("work-home");
        let work_runtime = directory.join("work-runtime");
        let personal_home = directory.join("personal-home");
        let personal_runtime = directory.join("personal-runtime");
        for path in [&work_home, &work_runtime, &personal_home, &personal_runtime] {
            fs::create_dir(path).expect("create isolation directory");
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))
                .expect("secure isolation directory");
        }
        let path = OsString::from("/usr/bin:/bin");
        let work =
            IsolatedClientEnvironment::new(work_home.clone(), work_runtime.clone(), path.clone())
                .expect("work environment");
        let personal =
            IsolatedClientEnvironment::new(personal_home.clone(), personal_runtime.clone(), path)
                .expect("personal environment");
        let work_output = run_bounded_isolated(
            &executable,
            std::iter::empty::<&OsStr>(),
            &work,
            Duration::from_secs(1),
            4096,
        )
        .expect("run work profile");
        let personal_output = run_bounded_isolated(
            &executable,
            std::iter::empty::<&OsStr>(),
            &personal,
            Duration::from_secs(1),
            4096,
        )
        .expect("run personal profile");
        let expected_work = format!(
            "{}|{}|{}|unset",
            work_home.display(),
            work_home.join(".local/share").display(),
            work_runtime.display()
        );
        let expected_personal = format!(
            "{}|{}|{}|unset",
            personal_home.display(),
            personal_home.join(".local/share").display(),
            personal_runtime.display()
        );
        assert_eq!(work_output.stdout, expected_work.as_bytes());
        assert_eq!(personal_output.stdout, expected_personal.as_bytes());
        assert_ne!(work_output.stdout, personal_output.stdout);
        remove(&directory);
    }

    #[test]
    fn isolated_environment_rejects_shared_or_linked_directories() {
        let (directory, _) = fixture("unused", "exit 0");
        let shared = directory.join("shared");
        let runtime = directory.join("runtime");
        fs::create_dir(&shared).expect("create shared directory");
        fs::create_dir(&runtime).expect("create runtime directory");
        fs::set_permissions(&shared, fs::Permissions::from_mode(0o755))
            .expect("set shared permissions");
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o700))
            .expect("set runtime permissions");
        let linked = directory.join("linked");
        symlink(&runtime, &linked).expect("create linked directory");
        assert!(matches!(
            IsolatedClientEnvironment::new(
                linked,
                runtime.clone(),
                OsString::from("/usr/bin:/bin")
            ),
            Err(ProcessError::UnsafeIsolation(_))
        ));
        assert!(matches!(
            IsolatedClientEnvironment::new(
                shared,
                runtime.clone(),
                OsString::from("/usr/bin:/bin")
            ),
            Err(ProcessError::UnsafeIsolation(_))
        ));
        assert!(matches!(
            IsolatedClientEnvironment::new(
                runtime.clone(),
                runtime,
                OsString::from("/usr/bin:/bin")
            ),
            Err(ProcessError::UnsafeIsolation(_))
        ));
        remove(&directory);
    }

    #[test]
    fn interactive_launcher_passes_argv_and_allowlisted_terminal_only() {
        let (directory, executable) = fixture(
            "interactive",
            "printf '%s|%s|%s|%s|%s' \"$2\" \"$HOME\" \"${TERM-unset}\" \"${USER-unset}\" \"${SSH_CONNECTION-unset}\" > \"$1\"; exit 23",
        );
        let home = directory.join("home");
        let runtime = directory.join("runtime");
        let observation = directory.join("observation");
        for path in [&home, &runtime] {
            fs::create_dir(path).expect("create isolation directory");
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))
                .expect("secure isolation directory");
        }
        let environment =
            IsolatedClientEnvironment::new(home.clone(), runtime, OsString::from("/usr/bin:/bin"))
                .expect("isolated environment")
                .with_terminal("xterm-256color")
                .expect("valid terminal")
                .with_ssh_file_fallback();
        let argument = "literal; shell syntax is data";
        let status = run_interactive_isolated(
            &executable,
            [observation.as_os_str(), OsStr::new(argument)],
            &environment,
        )
        .expect("run interactive fixture");
        assert_eq!(status.code(), Some(23));
        assert_eq!(
            fs::read_to_string(&observation).expect("read observation"),
            format!(
                "{argument}|{}|xterm-256color|unset|127.0.0.1 40000 127.0.0.1 22",
                home.display()
            )
        );
        assert!(!directory.join("shell syntax is data").exists());
        remove(&directory);
    }

    #[test]
    fn terminal_name_is_validated() {
        let (directory, _) = fixture("unused-terminal", "exit 0");
        let home = directory.join("home");
        let runtime = directory.join("runtime");
        for path in [&home, &runtime] {
            fs::create_dir(path).expect("create isolation directory");
            fs::set_permissions(path, fs::Permissions::from_mode(0o700))
                .expect("secure isolation directory");
        }
        let environment =
            IsolatedClientEnvironment::new(home, runtime, OsString::from("/usr/bin:/bin"))
                .expect("isolated environment");
        assert!(matches!(
            environment.with_terminal("xterm; injected"),
            Err(ProcessError::UnsafeIsolation(_))
        ));
        remove(&directory);
    }
}
