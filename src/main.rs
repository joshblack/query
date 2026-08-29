use std::process::ExitCode;

use clap::Parser;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(version, about = "Query source code relationships")]
struct Cli;

#[derive(Debug, Error)]
enum CliError {}

#[expect(
    clippy::unnecessary_wraps,
    reason = "the CLI error boundary is established before commands add fallible behavior"
)]
fn main() -> Result<ExitCode, CliError> {
    let _cli = Cli::parse();

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use clap::{CommandFactory, Parser, error::ErrorKind};

    use super::Cli;

    #[test]
    fn accepts_no_arguments() {
        assert!(Cli::try_parse_from(["query"]).is_ok());
    }

    #[test]
    fn defines_a_valid_command() {
        Cli::command().debug_assert();
    }

    #[test]
    fn displays_help() {
        assert!(matches!(
            Cli::try_parse_from(["query", "--help"]),
            Err(error) if error.kind() == ErrorKind::DisplayHelp
        ));
    }

    #[test]
    fn displays_version() {
        assert!(matches!(
            Cli::try_parse_from(["query", "--version"]),
            Err(error) if error.kind() == ErrorKind::DisplayVersion
        ));
    }
}
