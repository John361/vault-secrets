#[tokio::main]
async fn main() {
    init_tracing();
}

fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::debug!("Tracing initialized");
}
