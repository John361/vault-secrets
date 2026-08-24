use lib_vault_secrets::app::run;
use lib_vault_secrets::tracing::init_tracing;

#[tokio::main]
async fn main() {
    init_tracing();

    if let Err(e) = run().await {
        eprintln!("Error: {e:?}");
        std::process::exit(1);
    }
}
