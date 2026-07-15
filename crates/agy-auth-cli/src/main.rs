#![doc = "Command-line entry point for agy-auth."]

use agy_auth_app::{DoctorRegistryProbe, DoctorReport, RegistryDiagnostic, doctor};
use agy_auth_storage::RegistryDoctorProbe;
use clap::{Parser, Subcommand};
use provider_antigravity_cli::AntigravityDoctorProbe;
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

/// Supported diagnostics-only commands.
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
    };
    std::process::exit(exit_code.into());
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
    println!("profile switching: unsupported");
    println!("authentication mutation: disabled");
    println!("reason: no supported Antigravity profile contract is available");
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
