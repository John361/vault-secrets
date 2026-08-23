use lib_vault_secrets::cli::Cli;
use lib_vault_secrets::config::AppConfig;
use lib_vault_secrets::vault::VaultClient;

#[tokio::main]
async fn main() {
    init_tracing();

    let config = AppConfig::load("app.conf.yml").unwrap(); // TODO: load path with cli
    let vault_client = VaultClient::new(config.vault).await;

    let cli = Cli::load();

    match cli {
        Cli::Find(args) => {
            let result = vault_client.find(&args.path, &args.key).await.unwrap();
            tracing::info!("{:#?}", result);
        }
    }
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("vaultrs::client=warn".parse().unwrap())
                .add_directive("rustify=warn".parse().unwrap())
                .add_directive("reqwest=warn".parse().unwrap())
                .add_directive("hyper_util=warn".parse().unwrap())
        )
        .init();

    tracing::debug!("Tracing initialized");
}
