pub mod native;
pub mod sanctum;

use crate::error::Result;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signature::Signature};

#[async_trait]
pub trait PoolHandler: Send + Sync {
    async fn send_to_reserve(
        &self,
        rpc_client: &RpcClient,
        admin_keypair: &Keypair,
        reserve_address: &Pubkey,
        amount: u64,
    ) -> Result<Signature>;

    async fn crank_pool(
        &self,
        rpc_client: &RpcClient,
        pool_address: &Pubkey,
    ) -> Result<Option<Signature>>;

    async fn execute_crank_cycle(
        &self,
        rpc_client: &RpcClient,
        admin_keypair: &Keypair,
        pool_address: &Pubkey,
        reserve_address: &Pubkey,
        amount: u64,
    ) -> Result<(Signature, Option<Signature>)> {
        let deposit_sig = self
            .send_to_reserve(rpc_client, admin_keypair, reserve_address, amount)
            .await?;

        tracing::info!("Deposit transaction confirmed: {}", deposit_sig);

        rpc_client
            .confirm_transaction(&deposit_sig)
            .map_err(crate::error::CrankerError::Rpc)?;

        let crank_sig = self.crank_pool(rpc_client, pool_address).await?;

        if let Some(sig) = crank_sig {
            tracing::info!("Crank transaction confirmed: {}", sig);
        } else {
            tracing::info!("Crank not required (auto-registered)");
        }

        Ok((deposit_sig, crank_sig))
    }
}
