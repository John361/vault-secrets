use lib_vault_secrets::app::run;

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(e) = run().await {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
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
                .add_directive("hyper_util=warn".parse().unwrap()),
        )
        .init();

    tracing::debug!("Tracing initialized");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

    #[test]
    fn test_init_tracing() {
        let _ = panic::catch_unwind(init_tracing);
    }
}
