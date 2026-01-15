use crate::error::{CrankerError, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{signature::Keypair, signature::Signature, signer::keypair::keypair_from_seed};
use std::time::Duration;

pub fn parse_keypair(private_key: &str) -> Result<Keypair> {
    let decoded = bs58::decode(private_key)
        .into_vec()
        .map_err(|e| CrankerError::PrivateKey(format!("Failed to decode base58: {}", e)))?;

    if decoded.len() != 64 {
        return Err(CrankerError::PrivateKey(format!(
            "Invalid key length: expected 64 bytes, got {}",
            decoded.len()
        )));
    }

    let mut seed = [0u8; 32];
    seed.copy_from_slice(&decoded[0..32]);

    keypair_from_seed(&seed)
        .map_err(|e| CrankerError::PrivateKey(format!("Failed to create keypair: {}", e)))
}

pub async fn send_transaction_with_retry(
    rpc_client: &RpcClient,
    transaction: &solana_sdk::transaction::Transaction,
    max_retries: u32,
) -> Result<Signature> {
    let mut retries = 0;

    loop {
        match rpc_client.send_and_confirm_transaction(transaction) {
            Ok(sig) => return Ok(sig),
            Err(e) if retries < max_retries => {
                retries += 1;
                tracing::warn!(
                    "Transaction failed, retry {}/{}: {}",
                    retries,
                    max_retries,
                    e
                );
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
            Err(e) => return Err(CrankerError::Rpc(e)),
        }
    }
}

pub async fn confirm_transaction(rpc_client: &RpcClient, signature: &Signature) -> Result<()> {
    rpc_client
        .confirm_transaction(signature)
        .map_err(CrankerError::Rpc)?;
    Ok(())
}
