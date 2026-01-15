mod config;
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
        "Configuration loaded: pool_type={:?}, interval={:?}",
        config.pool_type,
        config.crank_interval
    );

    let scheduler = scheduler::CrankScheduler::new(config)?;

    scheduler.run().await?;

    Ok(())
}
