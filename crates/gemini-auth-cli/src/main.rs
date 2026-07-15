#![doc = "Command-line entry point for gemini-auth."]

use clap::Parser;
use std::path::PathBuf;

/// Local profile selection for credentials managed by official Google clients.
#[derive(Debug, Parser)]
#[command(name = "gemini-auth", version, about)]
struct Cli {
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

fn main() {
    let _cli = Cli::parse();
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
    }

    #[test]
    fn global_flags_parse() {
        let cli = Cli::try_parse_from(["gemini-auth", "--json", "--non-interactive"])
            .expect("documented global flags should parse");
        assert!(cli.json);
        assert!(cli.non_interactive);
    }
}
