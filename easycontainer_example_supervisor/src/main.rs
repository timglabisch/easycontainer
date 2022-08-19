use anyhow::anyhow;
use tracing::{debug, info};
use easycontainer_lib::process::service::run_services_from_glob;
use tracing_subscriber;


#[tokio::main]
async fn main() -> Result<(), ::anyhow::Error> {
    tracing_subscriber::fmt::init();

    run_services_from_glob("./services/*.toml").await?;

    Err(anyhow!("finished"))
}