#![doc = "Command-line entry point for agy-auth."]

#[cfg(feature = "profile-cli")]
use agy_auth_app::DoctorClientProbe;
use agy_auth_app::ProfileWorkflowError;
#[cfg(any(feature = "experimental-fake-client", feature = "profile-cli"))]
use agy_auth_app::new_pending_profile;
#[cfg(feature = "profile-cli")]
use agy_auth_app::{
    CredentialEnvelopePort, CredentialFilePort, CredentialSessionPorts, CredentialWorkflowError,
    OpaqueSecretBytes, ProfileCatalogPort, require_ready_profile, run_profile_credential_session,
};
use agy_auth_app::{DoctorRegistryProbe, DoctorReport, RegistryDiagnostic, doctor};
#[cfg(feature = "experimental-fake-client")]
use agy_auth_app::{ManagedProfileEnvironment, ProfileClientPort, add_profile, exec_profile};
#[cfg(any(feature = "experimental-fake-client", feature = "profile-cli"))]
use agy_auth_storage::ManagedProfileHomes;
#[cfg(feature = "profile-cli")]
use agy_auth_storage::{
    ActiveProfileStore, ImportTransactionJournal, ImportTransactionStage,
    OfficialCredentialSourceFiles, OpaqueCredentialBytes, ProfileCredentialFiles,
    ProfileSessionLock,
};
use agy_auth_storage::{RegistryCatalog, RegistryDoctorProbe};
use clap::{Parser, Subcommand};
use provider_antigravity_cli::AntigravityDoctorProbe;
#[cfg(feature = "profile-cli")]
use provider_antigravity_cli::{
    ANTIGRAVITY_TOKEN_RELATIVE_PATH, AntigravityCredentialEnvelope, AntigravityInteractiveSession,
};
#[cfg(any(feature = "experimental-fake-client", feature = "profile-cli"))]
use std::ffi::OsString;
use std::path::Path as StdPath;
use std::path::{Path, PathBuf};

/// Capability-gated local profile management for Google Antigravity CLI.
#[derive(Debug, Parser)]
#[command(name = "agy-auth", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Emit stable machine-readable output.
    #[arg(long, global = true)]
    json: bool,

    /// Disable ANSI styling.
    #[arg(long, global = true)]
    no_color: bool,

    /// Use an alternate non-secret configuration file.
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,

    /// Use an alternate data root.
    #[arg(long, global = true, value_name = "PATH")]
    data_dir: Option<PathBuf>,

    /// Refuse all interactive prompts.
    #[arg(long, global = true)]
    non_interactive: bool,

    /// Increase redacted diagnostic verbosity.
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
}

/// Supported profile-management and diagnostic commands.
#[derive(Debug, Subcommand)]
enum Commands {
    /// Diagnose official-client, registry, and capability status.
    Doctor {
        /// Validate repair eligibility; no authentication state is changed.
        #[arg(long)]
        repair: bool,

        /// Use an explicit Antigravity CLI executable.
        #[arg(long, value_name = "PATH")]
        client: Option<PathBuf>,
    },
    /// List registered profiles without reading authentication state.
    List,
    /// Save the account currently logged into the official agy home.
    #[cfg(feature = "profile-cli")]
    Add {
        /// Non-secret profile name; omitted names are generated automatically.
        name: Option<String>,
        /// Read from an alternate official agy home.
        #[arg(long, value_name = "PATH")]
        from_home: Option<PathBuf>,
        /// Use an explicit Antigravity CLI executable.
        #[arg(long, value_name = "PATH")]
        client: Option<PathBuf>,
    },
    /// Enroll another account through official agy in a new isolated home.
    #[cfg(feature = "profile-cli")]
    Login {
        /// Non-secret profile name; omitted names are generated automatically.
        name: Option<String>,
        /// Use an explicit Antigravity CLI executable.
        #[arg(long, value_name = "PATH")]
        client: Option<PathBuf>,
    },
    /// Select the account used by future plain agy launches.
    #[cfg(feature = "profile-cli")]
    Switch {
        /// Registered profile name.
        name: String,
        /// Use an explicit Antigravity CLI executable.
        #[arg(long, value_name = "PATH")]
        client: Option<PathBuf>,
    },
    /// Set a masked account hint used by list output.
    #[cfg(feature = "profile-cli")]
    Hint {
        /// Registered profile name.
        name: String,
        /// Masked hint such as a***@gmail.com.
        account_hint: String,
    },
    /// Exercise profile-add orchestration with an in-process fake client.
    #[cfg(feature = "experimental-fake-client")]
    #[command(hide = true)]
    ExperimentalAdd {
        /// Non-secret profile name.
        name: String,
    },
    /// Exercise profile-exec orchestration with an in-process fake client.
    #[cfg(feature = "experimental-fake-client")]
    #[command(hide = true)]
    ExperimentalExec {
        /// Registered profile name.
        name: String,
        /// Synthetic exit code returned by the fake client.
        #[arg(long, default_value_t = 0, value_parser = clap::value_parser!(u8))]
        fake_exit: u8,
        /// Literal fake-client arguments.
        #[arg(last = true)]
        arguments: Vec<OsString>,
    },
    /// Import a profile from a secure official agy home.
    #[cfg(feature = "profile-cli")]
    #[command(hide = true)]
    ExperimentalImport {
        /// Non-secret profile name.
        name: String,
        /// Absolute owner-only home containing an official agy credential file.
        #[arg(long, value_name = "PATH")]
        from_home: PathBuf,
        /// Use an explicit Antigravity CLI executable.
        #[arg(long, value_name = "PATH")]
        client: Option<PathBuf>,
    },
    /// Execute agy through a managed credential-backed profile.
    #[cfg(feature = "profile-cli")]
    Exec {
        /// Registered profile name.
        name: String,
        /// Use an explicit Antigravity CLI executable.
        #[arg(long, value_name = "PATH")]
        client: Option<PathBuf>,
        /// Literal arguments passed directly to agy.
        #[arg(last = true)]
        arguments: Vec<OsString>,
    },
    /// Recover interrupted profile imports.
    #[cfg(feature = "profile-cli")]
    Recover,
}

enum RegistryProbe {
    File(RegistryDoctorProbe),
    Unavailable,
}

impl DoctorRegistryProbe for RegistryProbe {
    fn probe(&self) -> RegistryDiagnostic {
        match self {
            Self::File(probe) => probe.probe(),
            Self::Unavailable => RegistryDiagnostic {
                healthy: false,
                profile_count: None,
                error_code: Some("registry_data_dir_unavailable".to_owned()),
                data_directory_state: "unsafe",
                owner_matches: None,
                permissions_secure: None,
                interrupted_transactions: 0,
            },
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let exit_code = match &cli.command {
        Commands::Doctor { repair, client } => run_doctor(&cli, client.clone(), *repair),
        Commands::List => run_list(&cli),
        #[cfg(feature = "profile-cli")]
        Commands::Add {
            name,
            from_home,
            client,
        } => run_add(
            &cli,
            name.as_deref(),
            from_home.as_deref(),
            client.as_deref(),
        ),
        #[cfg(feature = "profile-cli")]
        Commands::Login { name, client } => run_login(&cli, name.as_deref(), client.as_deref()),
        #[cfg(feature = "profile-cli")]
        Commands::Switch { name, client } => run_switch(&cli, name, client.as_deref()),
        #[cfg(feature = "profile-cli")]
        Commands::Hint { name, account_hint } => run_hint(&cli, name, account_hint),
        #[cfg(feature = "experimental-fake-client")]
        Commands::ExperimentalAdd { name } => run_experimental_add(&cli, name),
        #[cfg(feature = "experimental-fake-client")]
        Commands::ExperimentalExec {
            name,
            fake_exit,
            arguments,
        } => run_experimental_exec(&cli, name, *fake_exit, arguments),
        #[cfg(feature = "profile-cli")]
        Commands::ExperimentalImport {
            name,
            from_home,
            client,
        } => run_experimental_import(&cli, name, from_home, client.as_deref()),
        #[cfg(feature = "profile-cli")]
        Commands::Exec {
            name,
            client,
            arguments,
        } => run_experimental_real_exec(&cli, name, client.as_deref(), arguments),
        #[cfg(feature = "profile-cli")]
        Commands::Recover => run_recover(&cli),
    };
    std::process::exit(exit_code.into());
}

#[cfg(feature = "profile-cli")]
const VERIFIED_CLIENT_VERSIONS: [&str; 2] = ["1.1.2", "1.1.3"];

#[cfg(feature = "profile-cli")]
fn verified_client(
    explicit_client: Option<&Path>,
) -> Result<(Option<PathBuf>, OsString, String), RealProfileCliError> {
    if !cfg!(target_os = "linux") {
        return Err(RealProfileCliError::UnsupportedClient);
    }
    let search_path = std::env::var_os("PATH").ok_or(RealProfileCliError::ClientUnavailable)?;
    let explicit_client = explicit_client.map(Path::to_owned);
    let diagnostic =
        AntigravityDoctorProbe::new(explicit_client.clone(), Some(search_path.clone())).probe();
    if !diagnostic.found {
        return Err(RealProfileCliError::ClientUnavailable);
    }
    let version = diagnostic
        .version
        .filter(|version| VERIFIED_CLIENT_VERSIONS.contains(&version.as_str()))
        .ok_or(RealProfileCliError::UnsupportedClient)?;
    Ok((explicit_client, search_path, version))
}

#[cfg(feature = "profile-cli")]
#[derive(Clone, Copy)]
enum RealProfileCliError {
    ProfileConflict,
    ClientUnavailable,
    UnsupportedClient,
    UnsafeStorage,
    SessionBusy,
    ClientExecution,
    LoginFailed,
    Internal,
}

#[cfg(feature = "profile-cli")]
fn real_error_code(error: RealProfileCliError) -> u8 {
    match error {
        RealProfileCliError::ProfileConflict => 3,
        RealProfileCliError::ClientUnavailable => 4,
        RealProfileCliError::UnsupportedClient => 5,
        RealProfileCliError::UnsafeStorage => 8,
        RealProfileCliError::SessionBusy => 9,
        RealProfileCliError::LoginFailed => 6,
        RealProfileCliError::ClientExecution | RealProfileCliError::Internal => 11,
    }
}

#[cfg(feature = "profile-cli")]
fn real_error_message(error: RealProfileCliError) -> &'static str {
    match error {
        RealProfileCliError::ProfileConflict => "profile name is missing, pending, or already used",
        RealProfileCliError::ClientUnavailable => "official agy client is unavailable",
        RealProfileCliError::UnsupportedClient => {
            "installed agy version or platform is not verified for profile switching"
        }
        RealProfileCliError::UnsafeStorage => "profile storage or source permissions are unsafe",
        RealProfileCliError::SessionBusy => "profile is already running",
        RealProfileCliError::ClientExecution => "official agy client could not be executed",
        RealProfileCliError::LoginFailed => "official agy login did not complete successfully",
        RealProfileCliError::Internal => "profile operation failed an internal invariant",
    }
}

#[cfg(feature = "profile-cli")]
fn render_real_error(error: RealProfileCliError) -> u8 {
    eprintln!("agy-auth: {}", real_error_message(error));
    real_error_code(error)
}

#[cfg(feature = "profile-cli")]
fn map_credential_error(error: CredentialWorkflowError) -> RealProfileCliError {
    match error {
        CredentialWorkflowError::UnsupportedClientVersion => RealProfileCliError::UnsupportedClient,
        CredentialWorkflowError::CredentialStorageFailed => RealProfileCliError::UnsafeStorage,
        CredentialWorkflowError::SessionBusy => RealProfileCliError::SessionBusy,
        CredentialWorkflowError::ClientExecutionFailed => RealProfileCliError::ClientExecution,
        CredentialWorkflowError::InvalidSecretSize
        | CredentialWorkflowError::InvalidCredentialEnvelope => RealProfileCliError::Internal,
    }
}

#[cfg(feature = "profile-cli")]
fn run_experimental_import(
    cli: &Cli,
    name: &str,
    from_home: &Path,
    explicit_client: Option<&Path>,
) -> u8 {
    let result = (|| {
        let (_, _, client_version) = verified_client(explicit_client)?;
        let profile =
            new_pending_profile(name).map_err(|_| RealProfileCliError::ProfileConflict)?;
        let (catalog, homes) =
            experimental_adapters(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let root = experimental_data_root(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let journal =
            ImportTransactionJournal::new(root).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let mut transaction = journal
            .begin(profile.id)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let source = OfficialCredentialSourceFiles::new(from_home)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let source_envelope = source
            .read(Path::new(ANTIGRAVITY_TOKEN_RELATIVE_PATH), 16 * 1024)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let provider = AntigravityCredentialEnvelope;
        let refresh = provider
            .extract_refresh(
                &client_version,
                OpaqueSecretBytes::new(source_envelope.into_secret_bytes(), 16 * 1024)
                    .map_err(map_credential_error)?,
            )
            .map_err(map_credential_error)?;

        catalog
            .reserve(&profile)
            .map_err(|_| RealProfileCliError::ProfileConflict)?;
        transaction
            .advance(ImportTransactionStage::Reserved)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let environment = homes
            .prepare(profile.id)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let target = ProfileCredentialFiles::new(&environment.home)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        agy_auth_app::materialize_profile_credential(&client_version, &refresh, &provider, &target)
            .map_err(map_credential_error)?;
        transaction
            .advance(ImportTransactionStage::Materialized)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        catalog
            .mark_ready(profile.id, &client_version)
            .map_err(|_| RealProfileCliError::Internal)?;
        transaction
            .advance(ImportTransactionStage::Ready)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        transaction
            .complete()
            .map_err(|_| RealProfileCliError::UnsafeStorage)
    })();
    match result {
        Ok(()) => {
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "schemaVersion": 1,
                        "profile": name,
                        "status": "ready",
                    })
                );
            } else {
                println!("profile saved: {name}");
            }
            0
        }
        Err(error) => render_real_error(error),
    }
}

#[cfg(feature = "profile-cli")]
fn run_add(
    cli: &Cli,
    name: Option<&str>,
    from_home: Option<&Path>,
    explicit_client: Option<&Path>,
) -> u8 {
    let Some(home) = from_home
        .map(Path::to_owned)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
    else {
        return render_real_error(RealProfileCliError::UnsafeStorage);
    };
    let name = match resolve_profile_name(cli, name) {
        Ok(name) => name,
        Err(error) => return render_real_error(error),
    };
    run_experimental_import(cli, &name, &home, explicit_client)
}

#[cfg(feature = "profile-cli")]
fn run_login(cli: &Cli, requested_name: Option<&str>, explicit_client: Option<&Path>) -> u8 {
    if cli.non_interactive {
        return render_real_error(RealProfileCliError::LoginFailed);
    }
    let name = match resolve_profile_name(cli, requested_name) {
        Ok(name) => name,
        Err(error) => return render_real_error(error),
    };
    let result = (|| {
        let (explicit_client, search_path, client_version) = verified_client(explicit_client)?;
        let profile =
            new_pending_profile(&name).map_err(|_| RealProfileCliError::ProfileConflict)?;
        let (catalog, homes) =
            experimental_adapters(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let root = experimental_data_root(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let journal =
            ImportTransactionJournal::new(root).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let mut transaction = journal
            .begin(profile.id)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        catalog
            .reserve(&profile)
            .map_err(|_| RealProfileCliError::ProfileConflict)?;
        transaction
            .advance(ImportTransactionStage::Reserved)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let environment = homes
            .prepare(profile.id)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let client = AntigravityInteractiveSession::new(
            explicit_client.as_deref(),
            search_path,
            &environment,
            std::env::var("TERM").ok().as_deref(),
        )
        .map_err(map_credential_error)?;
        client
            .enroll_until_credential()
            .map_err(|_| RealProfileCliError::LoginFailed)?;
        let account_hint = client.masked_account_hint();
        let files = ProfileCredentialFiles::new(&environment.home)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        files
            .harden_credential_ancestors(Path::new(ANTIGRAVITY_TOKEN_RELATIVE_PATH))
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let provider = AntigravityCredentialEnvelope;
        let stored = CredentialFilePort::read(
            &files,
            Path::new(ANTIGRAVITY_TOKEN_RELATIVE_PATH),
            16 * 1024,
        )
        .map_err(map_credential_error)?;
        provider
            .extract_refresh(&client_version, stored)
            .map_err(map_credential_error)?;
        transaction
            .advance(ImportTransactionStage::Materialized)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        catalog
            .mark_ready(profile.id, &client_version)
            .map_err(|_| RealProfileCliError::Internal)?;
        if let Some(account_hint) = account_hint {
            catalog
                .set_account_hint(profile.id, Some(account_hint))
                .map_err(|_| RealProfileCliError::Internal)?;
        }
        transaction
            .advance(ImportTransactionStage::Ready)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        transaction
            .complete()
            .map_err(|_| RealProfileCliError::UnsafeStorage)
    })();
    match result {
        Ok(()) => {
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "schemaVersion": 1,
                        "profile": &name,
                        "status": "ready",
                    })
                );
            } else {
                println!("Successfully logged in: {name}");
                println!("Switch anytime: agy-auth switch {name}");
                println!("Help: agy-auth --help");
            }
            0
        }
        Err(error) => render_real_error(error),
    }
}

#[cfg(feature = "profile-cli")]
fn resolve_profile_name(cli: &Cli, requested: Option<&str>) -> Result<String, RealProfileCliError> {
    if let Some(name) = requested {
        return Ok(name.to_owned());
    }
    let catalog = catalog_adapter(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
    let profiles = catalog
        .profiles()
        .map_err(|_| RealProfileCliError::UnsafeStorage)?;
    for number in 1_u64.. {
        let candidate = format!("profile{number}");
        if profiles
            .iter()
            .all(|profile| profile.name.as_str() != candidate)
        {
            return Ok(candidate);
        }
    }
    unreachable!("u64 profile namespace cannot be exhausted")
}

#[cfg(feature = "profile-cli")]
fn run_recover(cli: &Cli) -> u8 {
    let result = (|| {
        let (catalog, homes) =
            experimental_adapters(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let root = experimental_data_root(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        ImportTransactionJournal::new(root)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?
            .recover(&catalog, &homes)
            .map_err(|_| RealProfileCliError::UnsafeStorage)
    })();
    match result {
        Ok(report) => {
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "schemaVersion": 1,
                        "rolledBack": report.rolled_back,
                        "completed": report.completed,
                    })
                );
            } else {
                println!(
                    "recovery complete: {} rolled back, {} completed",
                    report.rolled_back, report.completed
                );
            }
            0
        }
        Err(error) => render_real_error(error),
    }
}

fn run_list(cli: &Cli) -> u8 {
    let result = (|| {
        let catalog = catalog_adapter(cli)?;
        let selected = active_profile_store(cli)
            .map_err(|_| ProfileWorkflowError::HomeUnavailable)?
            .load()
            .map_err(|_| ProfileWorkflowError::HomeUnavailable)?;
        let mut profiles = catalog
            .profiles()
            .map_err(|_| ProfileWorkflowError::CatalogReserveFailed)?;
        profiles.sort_by(|left, right| left.name.as_str().cmp(right.name.as_str()));
        if cli.json {
            let values: Vec<_> = profiles
                .iter()
                .map(|profile| {
                    serde_json::json!({
                        "name": profile.name.as_str(),
                        "status": profile.status.as_str(),
                        "clientVersion": profile.client_version_at_capture,
                        "accountHint": profile.account_hint.as_deref(),
                        "selected": selected.as_deref() == Some(profile.name.as_str()),
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "schemaVersion": 1,
                    "profiles": values,
                }))
                .map_err(|_| ProfileWorkflowError::CatalogReserveFailed)?
            );
        } else if profiles.is_empty() {
            println!("no profiles");
        } else {
            for profile in profiles {
                println!(
                    "{} {}\t{}\t{}\t{}",
                    if selected.as_deref() == Some(profile.name.as_str()) {
                        "->"
                    } else {
                        "  "
                    },
                    profile.name.as_str(),
                    profile.status.as_str(),
                    profile.client_version_at_capture.as_deref().unwrap_or("-"),
                    profile.account_hint.as_deref().unwrap_or("-")
                );
            }
        }
        Ok(())
    })();
    match result {
        Ok(()) => 0,
        Err(ProfileWorkflowError::HomeUnavailable) => 8,
        Err(_) => 11,
    }
}

#[cfg(feature = "profile-cli")]
fn run_switch(cli: &Cli, name: &str, explicit_client: Option<&Path>) -> u8 {
    let result = (|| {
        let (_client, _search_path, client_version) = verified_client(explicit_client)?;
        let (catalog, homes) =
            experimental_adapters(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let target = catalog
            .profile(name)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?
            .ok_or(RealProfileCliError::ProfileConflict)?;
        require_ready_profile(&target).map_err(|_| RealProfileCliError::ProfileConflict)?;
        if target.client_version_at_capture.as_deref() != Some(client_version.as_str()) {
            return Err(RealProfileCliError::UnsupportedClient);
        }
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or(RealProfileCliError::UnsafeStorage)?;
        let official = OfficialCredentialSourceFiles::new(home)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let provider = AntigravityCredentialEnvelope;
        let active = active_profile_store(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let rollback = official
            .read(Path::new(ANTIGRAVITY_TOKEN_RELATIVE_PATH), 16 * 1024)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;

        if let Some(previous_name) = active
            .load()
            .map_err(|_| RealProfileCliError::UnsafeStorage)?
            .filter(|previous| previous != name)
        {
            let previous = catalog
                .profile(&previous_name)
                .map_err(|_| RealProfileCliError::UnsafeStorage)?
                .ok_or(RealProfileCliError::ProfileConflict)?;
            require_ready_profile(&previous).map_err(|_| RealProfileCliError::ProfileConflict)?;
            let current = official
                .read(Path::new(ANTIGRAVITY_TOKEN_RELATIVE_PATH), 16 * 1024)
                .map_err(|_| RealProfileCliError::UnsafeStorage)?;
            let refresh = provider
                .extract_refresh(
                    &client_version,
                    OpaqueSecretBytes::new(current.into_secret_bytes(), 16 * 1024)
                        .map_err(map_credential_error)?,
                )
                .map_err(map_credential_error)?;
            let previous_environment = homes
                .prepare(previous.id)
                .map_err(|_| RealProfileCliError::UnsafeStorage)?;
            let previous_files = ProfileCredentialFiles::new(previous_environment.home)
                .map_err(|_| RealProfileCliError::UnsafeStorage)?;
            let plan = provider
                .build_plan(&client_version, &refresh)
                .map_err(map_credential_error)?;
            CredentialFilePort::materialize(&previous_files, &plan.relative_path, &plan.envelope)
                .map_err(map_credential_error)?;
        }

        let target_environment = homes
            .prepare(target.id)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let target_files = ProfileCredentialFiles::new(target_environment.home)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let stored = CredentialFilePort::read(
            &target_files,
            Path::new(ANTIGRAVITY_TOKEN_RELATIVE_PATH),
            16 * 1024,
        )
        .map_err(map_credential_error)?;
        let refresh = provider
            .extract_refresh(&client_version, stored)
            .map_err(map_credential_error)?;
        let plan = provider
            .build_plan(&client_version, &refresh)
            .map_err(map_credential_error)?;
        let envelope = OpaqueCredentialBytes::new(plan.envelope.into_secret_bytes(), 16 * 1024)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        official
            .materialize(&plan.relative_path, &envelope)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        if active.save(name).is_err() {
            official
                .materialize(&plan.relative_path, &rollback)
                .map_err(|_| RealProfileCliError::UnsafeStorage)?;
            return Err(RealProfileCliError::UnsafeStorage);
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({"schemaVersion": 1, "profile": name, "selected": true})
                );
            } else {
                println!("Switched to {name}. Run `agy` to start.");
            }
            0
        }
        Err(error) => render_real_error(error),
    }
}

#[cfg(feature = "profile-cli")]
fn run_hint(cli: &Cli, name: &str, account_hint: &str) -> u8 {
    let result = (|| {
        if !account_hint.contains('*') || !account_hint.contains('@') {
            return Err(RealProfileCliError::ProfileConflict);
        }
        let catalog = catalog_adapter(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let profile = catalog
            .profile(name)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?
            .ok_or(RealProfileCliError::ProfileConflict)?;
        catalog
            .set_account_hint(profile.id, Some(account_hint.to_owned()))
            .map_err(|_| RealProfileCliError::ProfileConflict)
    })();
    match result {
        Ok(()) => {
            if cli.json {
                println!(
                    "{}",
                    serde_json::json!({
                        "schemaVersion": 1,
                        "profile": name,
                        "accountHint": account_hint,
                    })
                );
            } else {
                println!("account hint updated: {name}");
            }
            0
        }
        Err(error) => render_real_error(error),
    }
}

#[cfg(feature = "profile-cli")]
fn run_experimental_real_exec(
    cli: &Cli,
    name: &str,
    explicit_client: Option<&Path>,
    arguments: &[OsString],
) -> u8 {
    let result = (|| {
        let (explicit_client, search_path, client_version) = verified_client(explicit_client)?;
        let (catalog, homes) =
            experimental_adapters(cli).map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let profile = catalog
            .profile(name)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?
            .ok_or(RealProfileCliError::ProfileConflict)?;
        require_ready_profile(&profile).map_err(|_| RealProfileCliError::ProfileConflict)?;
        if profile.client_version_at_capture.as_deref() != Some(client_version.as_str()) {
            return Err(RealProfileCliError::UnsupportedClient);
        }
        let environment = homes
            .prepare(profile.id)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let files = ProfileCredentialFiles::new(&environment.home)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let provider = AntigravityCredentialEnvelope;
        let stored = CredentialFilePort::read(
            &files,
            Path::new(ANTIGRAVITY_TOKEN_RELATIVE_PATH),
            16 * 1024,
        )
        .map_err(map_credential_error)?;
        let refresh = provider
            .extract_refresh(&client_version, stored)
            .map_err(map_credential_error)?;
        let client = AntigravityInteractiveSession::new(
            explicit_client.as_deref(),
            search_path,
            &environment,
            std::env::var("TERM").ok().as_deref(),
        )
        .map_err(map_credential_error)?;
        let lock = ProfileSessionLock::new(&environment.runtime_directory)
            .map_err(|_| RealProfileCliError::UnsafeStorage)?;
        let ports = CredentialSessionPorts {
            envelope: &provider,
            files: &files,
            client: &client,
            lock: &lock,
        };
        run_profile_credential_session(&client_version, &refresh, arguments, &ports, 16 * 1024)
            .map(|outcome| outcome.exit_code)
            .map_err(map_credential_error)
    })();
    match result {
        Ok(code) => u8::try_from(code).unwrap_or(11),
        Err(error) => render_real_error(error),
    }
}

#[cfg(feature = "experimental-fake-client")]
struct InProcessFakeClient {
    exit_code: u8,
}

#[cfg(feature = "experimental-fake-client")]
impl ProfileClientPort for InProcessFakeClient {
    fn login(
        &self,
        _environment: &ManagedProfileEnvironment,
    ) -> Result<String, ProfileWorkflowError> {
        Ok("TEST_FAKE_CLIENT_1.1.2".to_owned())
    }

    fn execute(
        &self,
        _environment: &ManagedProfileEnvironment,
        _arguments: &[OsString],
    ) -> Result<i32, ProfileWorkflowError> {
        Ok(i32::from(self.exit_code))
    }
}

#[cfg(any(feature = "experimental-fake-client", feature = "profile-cli"))]
fn experimental_adapters(
    cli: &Cli,
) -> Result<(RegistryCatalog, ManagedProfileHomes), ProfileWorkflowError> {
    let root = experimental_data_root(cli)?;
    let parent = root.parent().ok_or(ProfileWorkflowError::HomeUnavailable)?;
    if !StdPath::new(parent).is_dir() {
        return Err(ProfileWorkflowError::HomeUnavailable);
    }
    let catalog = catalog_adapter(cli)?;
    let homes =
        ManagedProfileHomes::new(root).map_err(|_| ProfileWorkflowError::HomeUnavailable)?;
    homes
        .initialize()
        .map_err(|_| ProfileWorkflowError::HomeUnavailable)?;
    Ok((catalog, homes))
}

fn catalog_adapter(cli: &Cli) -> Result<RegistryCatalog, ProfileWorkflowError> {
    let root = experimental_data_root(cli)?;
    let parent = root.parent().ok_or(ProfileWorkflowError::HomeUnavailable)?;
    if !StdPath::new(parent).is_dir() {
        return Err(ProfileWorkflowError::HomeUnavailable);
    }
    RegistryCatalog::new(root.join("registry.json"))
        .map_err(|_| ProfileWorkflowError::CatalogReserveFailed)
}

fn experimental_data_root(cli: &Cli) -> Result<PathBuf, ProfileWorkflowError> {
    cli.data_dir
        .clone()
        .or_else(default_data_dir)
        .ok_or(ProfileWorkflowError::HomeUnavailable)
}

fn active_profile_store(cli: &Cli) -> Result<ActiveProfileStore, ProfileWorkflowError> {
    let root = experimental_data_root(cli)?;
    ActiveProfileStore::new(root.join("active.json"))
        .map_err(|_| ProfileWorkflowError::HomeUnavailable)
}

#[cfg(feature = "experimental-fake-client")]
fn run_experimental_add(cli: &Cli, name: &str) -> u8 {
    let result = (|| {
        let profile = new_pending_profile(name)?;
        let (catalog, homes) = experimental_adapters(cli)?;
        add_profile(
            &profile,
            &catalog,
            &homes,
            &InProcessFakeClient { exit_code: 0 },
        )
    })();
    match result {
        Ok(()) => 0,
        Err(ProfileWorkflowError::InvalidProfile | ProfileWorkflowError::CatalogReserveFailed) => 3,
        Err(ProfileWorkflowError::HomeUnavailable) => 8,
        Err(_) => 11,
    }
}

#[cfg(feature = "experimental-fake-client")]
fn run_experimental_exec(cli: &Cli, name: &str, fake_exit: u8, arguments: &[OsString]) -> u8 {
    let result = (|| {
        let (catalog, homes) = experimental_adapters(cli)?;
        let profile = catalog
            .profile(name)
            .map_err(|_| ProfileWorkflowError::CatalogReserveFailed)?
            .ok_or(ProfileWorkflowError::ProfileNotReady)?;
        exec_profile(
            &profile,
            arguments,
            &homes,
            &InProcessFakeClient {
                exit_code: fake_exit,
            },
        )
    })();
    match result {
        Ok(code) => u8::try_from(code).unwrap_or(11),
        Err(ProfileWorkflowError::ProfileNotReady) => 3,
        Err(ProfileWorkflowError::HomeUnavailable) => 8,
        Err(_) => 11,
    }
}

fn run_doctor(cli: &Cli, explicit_client: Option<PathBuf>, repair: bool) -> u8 {
    let client = AntigravityDoctorProbe::new(explicit_client, std::env::var_os("PATH"));
    let registry = registry_probe(cli.data_dir.as_deref());
    let report = doctor(&client, &registry);
    if cli.json {
        match serde_json::to_string_pretty(&report) {
            Ok(json) => println!("{json}"),
            Err(_) => return 11,
        }
    } else {
        render_human(&report, repair);
    }
    report.exit_code()
}

fn registry_probe(data_dir: Option<&Path>) -> RegistryProbe {
    let root = data_dir.map(Path::to_owned).or_else(default_data_dir);
    root.and_then(|root| RegistryDoctorProbe::new(root.join("registry.json")).ok())
        .map_or(RegistryProbe::Unavailable, RegistryProbe::File)
}

fn default_data_dir() -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("XDG_DATA_HOME") {
        return Some(PathBuf::from(root).join("agy-auth"));
    }
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("agy-auth")
    })
}

fn render_human(report: &DoctorReport, repair: bool) {
    println!("agy-auth doctor");
    match (&report.client.version, &report.client.error_code) {
        (Some(version), _) => println!("client: agy {version}"),
        (None, Some(code)) => println!("client: unavailable ({code})"),
        (None, None) => println!("client: unavailable"),
    }
    if report.registry.healthy {
        println!(
            "registry: healthy ({} profiles)",
            report.registry.profile_count.unwrap_or(0)
        );
    } else {
        println!(
            "registry: unhealthy ({})",
            report.registry.error_code.as_deref().unwrap_or("unknown")
        );
    }
    println!("data directory: {}", report.registry.data_directory_state);
    println!(
        "interrupted transactions: {}",
        report.registry.interrupted_transactions
    );
    println!(
        "profile switching: {}",
        if report.capabilities.profile_switching {
            "supported"
        } else {
            "unsupported"
        }
    );
    println!(
        "authentication mutation: {}",
        if report.capabilities.auth_state_mutation {
            "enabled"
        } else {
            "disabled"
        }
    );
    println!("reason: {}", report.capabilities.reason);
    if repair {
        println!("repair: no safe repair actions are available");
    }
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::{CommandFactory, Parser};

    #[test]
    fn command_definition_is_valid() {
        Cli::command().debug_assert();
        let help = Cli::command().render_long_help().to_string();
        assert!(help.contains("--json"));
        assert!(help.contains("--non-interactive"));
        assert!(help.contains("doctor"));
    }

    #[test]
    fn global_flags_parse() {
        let cli = Cli::try_parse_from(["agy-auth", "--json", "--non-interactive", "doctor"])
            .expect("documented global flags should parse");
        assert!(cli.json);
        assert!(cli.non_interactive);
    }

    #[test]
    fn doctor_options_parse() {
        let cli = Cli::try_parse_from(["agy-auth", "doctor", "--repair", "--client", "/tmp/agy"])
            .expect("doctor options should parse");
        assert!(matches!(
            cli.command,
            super::Commands::Doctor {
                repair: true,
                client: Some(_)
            }
        ));
    }
}
