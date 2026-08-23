use lib_vault_secrets::config::AppConfig;

#[tokio::main]
async fn main() {
    init_tracing();

    let _config = AppConfig::load("app.conf.yml").unwrap(); // TODO: load path with cli
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::debug!("Tracing initialized");
}
