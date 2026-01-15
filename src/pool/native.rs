use crate::error::{CrankerError, Result};
use crate::pool::PoolHandler;
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_program::borsh0_10::try_from_slice_unchecked;
use solana_sdk::{
    pubkey::Pubkey, signature::Keypair, signature::Signature, signer::Signer, system_instruction,
    transaction::Transaction,
};
use spl_stake_pool::state::StakePool;

pub struct NativePoolHandler;

impl NativePoolHandler {
    pub fn new() -> Self {
        Self
    }

    fn build_update_instruction(
        stake_pool_address: &Pubkey,
        stake_pool: &StakePool,
    ) -> Result<solana_sdk::instruction::Instruction> {
        let withdraw_authority = spl_stake_pool::find_withdraw_authority_program_address(
            &spl_stake_pool::id(),
            stake_pool_address,
        )
        .0;

        let instruction = spl_stake_pool::instruction::update_stake_pool_balance(
            &spl_stake_pool::id(),
            stake_pool_address,
            &withdraw_authority,
            &stake_pool.validator_list,
            &stake_pool.reserve_stake,
            &stake_pool.manager_fee_account,
            &stake_pool.pool_mint,
            &spl_token::id(),
        );

        Ok(instruction)
    }
}

impl Default for NativePoolHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PoolHandler for NativePoolHandler {
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
            .map_err(CrankerError::Rpc)?;

        transaction.sign(&[admin_keypair], recent_blockhash);

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(CrankerError::Rpc)?;

        tracing::info!(
            "Native SPL: Sent {} lamports to reserve {} with signature {}",
            amount,
            reserve_address,
            signature
        );

        Ok(signature)
    }

    async fn crank_pool(
        &self,
        rpc_client: &RpcClient,
        pool_address: &Pubkey,
    ) -> Result<Option<Signature>> {
        let account_data = rpc_client
            .get_account_data(pool_address)
            .map_err(CrankerError::Rpc)?;

        let stake_pool: StakePool = try_from_slice_unchecked(&account_data).map_err(|e| {
            CrankerError::Pool(format!("Failed to deserialize stake pool: {}", e))
        })?;

        let update_ix = Self::build_update_instruction(pool_address, &stake_pool)?;

        let mut transaction = Transaction::new_with_payer(&[update_ix], None);

        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .map_err(CrankerError::Rpc)?;

        let signers: Vec<&dyn Signer> = vec![];
        transaction.sign(&signers, recent_blockhash);

        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .map_err(CrankerError::Rpc)?;

        tracing::info!(
            "Native SPL: Updated stake pool balance with signature {}",
            signature
        );

        Ok(Some(signature))
    }
}
