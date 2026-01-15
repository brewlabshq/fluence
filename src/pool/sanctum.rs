use crate::error::Result;
use crate::pool::PoolHandler;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signature::Signature, signer::Signer, system_instruction,
    transaction::Transaction,
};

pub struct SanctumPoolHandler;

impl SanctumPoolHandler {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SanctumPoolHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PoolHandler for SanctumPoolHandler {
    async fn send_to_reserve(
        &self,
        rpc_client: &RpcClient,
        admin_keypair: &Keypair,
        reserve_address: &Pubkey,
        amount: u64,
    ) -> Result<Signature> {
        let instruction =
            system_instruction::transfer(&admin_keypair.pubkey(), reserve_address, amount);

        let mut transaction =
            Transaction::new_with_payer(&[instruction], Some(&admin_keypair.pubkey()));

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .map_err(crate::error::CrankerError::Rpc)?;

        transaction.sign(&[admin_keypair], recent_blockhash);

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(crate::error::CrankerError::Rpc)?;

        tracing::info!(
            "Sanctum: Sent {} lamports to reserve {} with signature {}",
            amount,
            reserve_address,
            signature
        );

        Ok(signature)
    }

    async fn crank_pool(
        &self,
        _rpc_client: &RpcClient,
        _pool_address: &Pubkey,
    ) -> Result<Option<Signature>> {
        tracing::info!("Sanctum: Pool cranking not required (deposits are auto-registered)");
        Ok(None)
    }
}
