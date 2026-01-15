use crate::config::{CrankerConfig, PoolType};
use crate::error::Result;
use crate::pool::{native::NativePoolHandler, sanctum::SanctumPoolHandler, PoolHandler};
use crate::transaction;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::str::FromStr;

pub struct CrankScheduler {
    config: CrankerConfig,
    rpc_client: RpcClient,
    pool_handler: Box<dyn PoolHandler>,
    admin_keypair: Keypair,
}

impl CrankScheduler {
    pub fn new(config: CrankerConfig) -> Result<Self> {
        let rpc_client = RpcClient::new(config.rpc_url.clone());

        let pool_handler: Box<dyn PoolHandler> = match config.pool_type {
            PoolType::Sanctum => Box::new(SanctumPoolHandler::new()),
            PoolType::Native => Box::new(NativePoolHandler::new()),
        };

        let admin_keypair = transaction::parse_keypair(&config.admin_private_key)?;

        tracing::info!(
            "Initialized cranker with admin pubkey: {}",
            admin_keypair.pubkey()
        );

        Ok(Self {
            config,
            rpc_client,
            pool_handler,
            admin_keypair,
        })
    }

    pub async fn run(&self) -> Result<()> {
        tracing::info!(
            "Starting crank scheduler with interval: {:?}",
            self.config.crank_interval
        );

        let mut interval = tokio::time::interval(self.config.crank_interval);

        loop {
            interval.tick().await;

            tracing::info!("Starting crank cycle...");

            match self.execute_crank().await {
                Ok((deposit_sig, crank_sig)) => {
                    if let Some(sig) = crank_sig {
                        tracing::info!(
                            "Crank cycle completed successfully: deposit={}, crank={}",
                            deposit_sig,
                            sig
                        );
                    } else {
                        tracing::info!(
                            "Crank cycle completed successfully: deposit={} (crank not required)",
                            deposit_sig
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Crank cycle failed: {}", e);
                }
            }
        }
    }

    async fn execute_crank(
        &self,
    ) -> Result<(
        solana_sdk::signature::Signature,
        Option<solana_sdk::signature::Signature>,
    )> {
        let reserve_address = Pubkey::from_str(&self.config.pool_reserve_address).map_err(|e| {
            crate::error::CrankerError::Config(format!("Invalid reserve address: {}", e))
        })?;

        let pool_address = if let Some(ref pool_addr) = self.config.pool_address {
            Pubkey::from_str(pool_addr).map_err(|e| {
                crate::error::CrankerError::Config(format!("Invalid pool address: {}", e))
            })?
        } else {
            reserve_address
        };

        self.pool_handler
            .execute_crank_cycle(
                &self.rpc_client,
                &self.admin_keypair,
                &pool_address,
                &reserve_address,
                self.config.crank_amount,
            )
            .await
    }
}
