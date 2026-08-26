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

    #[command(about = "Import secrets")]
    Import(ImportArgs),
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

    #[arg(long, help = "Output folder path (required)", required = true)]
    pub output_folder: PathBuf,
}

#[derive(Args, Debug)]
#[command(about = "Import arguments", long_about = None)]
pub struct ImportArgs {
    #[arg(long, help = "Path to secret (required)", required = true)]
    pub path: String,

    #[arg(long, help = "Input folder path (required)", required = true)]
    pub input_folder: PathBuf,
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

        if let Commands::Find(args) = &cli.command {
            assert_eq!(args.path, "secret/data/mysql");
            assert_eq!(args.key, "password");
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
            "--output-folder",
            "./tests",
        ];

        let cli = Cli::try_load_from(args).unwrap();
        assert_eq!(cli.config, "/tmp/config.yaml");

        if let Commands::Export(args) = &cli.command {
            assert_eq!(args.path, "secret");
            assert_eq!(args.output_folder, PathBuf::from("./tests"));
        }
    }

    #[test]
    fn test_cli_load_import_success() {
        let args = vec![
            "vault-secrets",
            "--config",
            "/tmp/config.yaml",
            "import",
            "--path",
            "secret",
            "--input-folder",
            "./tests",
        ];

        let cli = Cli::try_load_from(args).unwrap();
        assert_eq!(cli.config, "/tmp/config.yaml");

        if let Commands::Import(args) = &cli.command {
            assert_eq!(args.path, "secret");
            assert_eq!(args.input_folder, PathBuf::from("./tests"));
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
    fn test_cli_missing_import_subcommand_args() {
        let args = vec!["vault-secrets", "--config", "/tmp/config.yaml", "import"];
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
        assert!(!cli.clear_output);

        if let Commands::Find(args) = cli.command {
            assert_eq!(args.path, "secret/data/mysql");
            assert_eq!(args.key, "password");
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
            "--output-folder",
            "./tests",
        ];

        let cli = Cli::try_load_from(args).unwrap();
        assert_eq!(cli.config, "/tmp/config.yaml");
        assert!(cli.clear_output);

        if let Commands::Export(args) = cli.command {
            assert_eq!(args.path, "secret");
            assert_eq!(args.output_folder, PathBuf::from("./tests"));
        }
    }

    #[test]
    fn test_cli_load_import_uses_parse() {
        let args = vec![
            "vault-secrets",
            "--clear-output",
            "--config",
            "/tmp/config.yaml",
            "import",
            "--path",
            "secret",
            "--input-folder",
            "./tests",
        ];

        let cli = Cli::try_load_from(args).unwrap();
        assert_eq!(cli.config, "/tmp/config.yaml");
        assert!(cli.clear_output);

        if let Commands::Import(args) = cli.command {
            assert_eq!(args.path, "secret");
            assert_eq!(args.input_folder, PathBuf::from("./tests"));
        }
    }
}
