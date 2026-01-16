use crate::config::{CrankerConfig, PoolType};
use crate::epoch_state::EpochState;
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
    epoch_state: EpochState,
}

impl CrankScheduler {
    pub fn new(config: CrankerConfig) -> Result<Self> {
        let rpc_client = RpcClient::new(config.rpc_url.clone());

        let pool_handler: Box<dyn PoolHandler> = match config.pool_type {
            PoolType::Sanctum => Box::new(SanctumPoolHandler::new()),
            PoolType::Native => Box::new(NativePoolHandler::new()),
        };

        let admin_keypair = transaction::parse_keypair(&config.admin_private_key)?;

        let epoch_state = EpochState::new(
            config.epoch_storage_type.clone(),
            config.epoch_state_file.clone(),
        );

        tracing::info!(
            "Initialized cranker with admin pubkey: {}",
            admin_keypair.pubkey()
        );

        Ok(Self {
            config,
            rpc_client,
            pool_handler,
            admin_keypair,
            epoch_state,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        tracing::info!(
            "Starting epoch-based crank scheduler (polling every {:?})",
            self.config.epoch_poll_interval
        );

        // Load last cranked epoch from storage
        let last_cranked_epoch = match self.epoch_state.load() {
            Ok(epoch) => {
                if let Some(e) = epoch {
                    tracing::info!("Restored last cranked epoch: {}", e);
                }
                epoch
            }
            Err(e) => {
                tracing::warn!("Failed to load epoch state, starting fresh: {}", e);
                None
            }
        };

        let mut last_cranked_epoch = last_cranked_epoch;
        let mut interval = tokio::time::interval(self.config.epoch_poll_interval);

        loop {
            interval.tick().await;

            let current_epoch = match self.rpc_client.get_epoch_info() {
                Ok(info) => info.epoch,
                Err(e) => {
                    tracing::error!("Failed to get epoch info: {}", e);
                    continue;
                }
            };

            let should_crank = match last_cranked_epoch {
                Some(last_epoch) => current_epoch > last_epoch,
                None => true,
            };

            if !should_crank {
                tracing::debug!(
                    "Epoch {} already cranked, waiting for next epoch",
                    current_epoch
                );
                continue;
            }

            tracing::info!("New epoch detected: {}. Starting crank cycle...", current_epoch);

            match self.execute_crank().await {
                Ok((deposit_sig, crank_sig)) => {
                    last_cranked_epoch = Some(current_epoch);
                    if let Err(e) = self.epoch_state.save(current_epoch) {
                        tracing::error!("Failed to save epoch state: {}", e);
                    }
                    if let Some(sig) = crank_sig {
                        tracing::info!(
                            "Crank cycle completed for epoch {}: deposit={}, crank={}",
                            current_epoch,
                            deposit_sig,
                            sig
                        );
                    } else {
                        tracing::info!(
                            "Crank cycle completed for epoch {}: deposit={} (crank not required)",
                            current_epoch,
                            deposit_sig
                        );
                    }
                }
                Err(e) => {
                    tracing::error!("Crank cycle failed for epoch {}: {}", current_epoch, e);
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
