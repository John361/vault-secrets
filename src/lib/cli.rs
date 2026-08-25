use std::path::PathBuf;

use clap::Parser;
use clap::{Args, ValueEnum};

#[derive(Debug, Parser)]
#[command(version, name = "vault-secrets", bin_name = "vault-secrets")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(
        long = "config",
        required = true,
        help = "Path to config file (required)"
    )]
    pub config: String,

    #[arg(
        long = "clear-output",
        help = "Encode output data (optional, default: false)"
    )]
    pub clear_output: bool,
}

impl Cli {
    pub fn load() -> Self {
        Self::try_load_from(std::env::args_os()).unwrap_or_else(|e| e.exit())
    }

    pub fn try_load_from<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Cli::try_parse_from(args)
    }
}

#[derive(Debug, Parser)]
pub enum Commands {
    #[command(about = "Find secret")]
    Find(FindArgs),

    #[command(about = "Export secrets")]
    Export(ExportArgs),
}

#[derive(Args, Debug)]
#[command(about = "Find arguments", long_about = None)]
pub struct FindArgs {
    #[arg(long, help = "Path to secret (required)", required = true)]
    pub path: String,

    #[arg(long, help = "Secret key name (required)", required = true)]
    pub key: String,
}

#[derive(Args, Debug)]
#[command(about = "Export arguments", long_about = None)]
pub struct ExportArgs {
    #[arg(long, help = "Path to secret (required)", required = true)]
    pub path: String,

    #[arg(long, help = "Output file path (required)", required = true)]
    pub output_file: PathBuf,

    #[arg(
        long,
        help = "Output format (optional, default: json)",
        default_value_t = FormatArgs::Json,
        value_enum
    )]
    pub output_format: FormatArgs,
}

#[derive(Clone, Debug, Default, PartialEq, ValueEnum)]
pub enum FormatArgs {
    #[default]
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::error::ErrorKind;

    #[test]
    fn test_cli_load_find_success() {
        let args = vec![
            "vault-secrets",
            "--config",
            "/tmp/config.yaml",
            "find",
            "--path",
            "secret/data/mysql",
            "--key",
            "password",
        ];

        let cli = Cli::try_load_from(args).unwrap();
        assert_eq!(cli.config, "/tmp/config.yaml");

        match cli.command {
            Commands::Find(find_args) => {
                assert_eq!(find_args.path, "secret/data/mysql");
                assert_eq!(find_args.key, "password");
            }
            _ => {}
        }
    }

    #[test]
    fn test_cli_load_export_success() {
        let args = vec![
            "vault-secrets",
            "--config",
            "/tmp/config.yaml",
            "export",
            "--path",
            "secret",
            "--output-file",
            "./secret.json",
        ];

        let cli = Cli::try_load_from(args).unwrap();
        assert_eq!(cli.config, "/tmp/config.yaml");

        match cli.command {
            Commands::Export(find_args) => {
                assert_eq!(find_args.path, "secret");
                assert_eq!(find_args.output_file, PathBuf::from("./secret.json"));
                assert_eq!(find_args.output_format, FormatArgs::Json);
            }
            _ => {}
        }
    }

    #[test]
    fn test_cli_missing_config() {
        let args = vec![
            "vault-secrets",
            "find",
            "--path",
            "secret/data/mysql",
            "--key",
            "password",
        ];

        let err = Cli::try_load_from(args).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn test_cli_missing_find_subcommand_args() {
        let args = vec!["vault-secrets", "--config", "/tmp/config.yaml", "find"];
        let err = Cli::try_load_from(args).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn test_cli_missing_export_subcommand_args() {
        let args = vec!["vault-secrets", "--config", "/tmp/config.yaml", "export"];
        let err = Cli::try_load_from(args).unwrap_err();

        assert_eq!(err.kind(), ErrorKind::MissingRequiredArgument);
    }

    #[test]
    fn test_cli_unknown_command() {
        let args = vec![
            "vault-secrets",
            "--config",
            "/tmp/config.yaml",
            "unknown-command",
        ];

        let err = Cli::try_load_from(args).unwrap_err();
        assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn test_cli_load_find_uses_parse() {
        let args = vec![
            "vault-secrets",
            "--config",
            "/tmp/config.yaml",
            "find",
            "--path",
            "secret/data/mysql",
            "--key",
            "password",
        ];

        let cli = Cli::try_load_from(args).unwrap();
        assert_eq!(cli.config, "/tmp/config.yaml");
        assert_eq!(cli.clear_output, false);

        match cli.command {
            Commands::Find(find_args) => {
                assert_eq!(find_args.path, "secret/data/mysql");
                assert_eq!(find_args.key, "password");
            }
            _ => {}
        }
    }

    #[test]
    fn test_cli_load_export_uses_parse() {
        let args = vec![
            "vault-secrets",
            "--clear-output",
            "--config",
            "/tmp/config.yaml",
            "export",
            "--path",
            "secret",
            "--output-file",
            "./secrets.json",
        ];

        let cli = Cli::try_load_from(args).unwrap();
        assert_eq!(cli.config, "/tmp/config.yaml");
        assert_eq!(cli.clear_output, true);

        match cli.command {
            Commands::Export(find_args) => {
                assert_eq!(find_args.path, "secret");
                assert_eq!(find_args.output_file, PathBuf::from("./secrets.json"));
            }
            _ => {}
        }
    }
}
