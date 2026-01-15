use thiserror::Error;

#[derive(Debug, Error)]
pub enum CrankerError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("RPC error: {0}")]
    Rpc(#[from] solana_client::client_error::ClientError),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Pool operation error: {0}")]
    Pool(String),

    #[error("Invalid pool type: {0}")]
    InvalidPoolType(String),

    #[error("Private key error: {0}")]
    PrivateKey(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),
}

pub type Result<T> = std::result::Result<T, CrankerError>;
