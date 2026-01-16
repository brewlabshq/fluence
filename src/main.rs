mod config;
mod epoch_state;
mod error;
mod pool;
mod scheduler;
mod transaction;

use error::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "fluence=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv::dotenv().ok();

    tracing::info!("Starting Solana Stake Pool Cranker");

    let config = config::CrankerConfig::load()?;

    tracing::info!(
        "Configuration loaded: pool_type={:?}, epoch_poll_interval={:?}, epoch_storage={:?}",
        config.pool_type,
        config.epoch_poll_interval,
        config.epoch_storage_type
    );

    let mut scheduler = scheduler::CrankScheduler::new(config)?;

    scheduler.run().await?;

    Ok(())
}
