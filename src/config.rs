use crate::error::{CrankerError, Result};
use std::env;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PoolType {
    Sanctum,
    Native,
}

impl PoolType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "sanctum" => Ok(PoolType::Sanctum),
            "native" => Ok(PoolType::Native),
            _ => Err(CrankerError::InvalidPoolType(format!(
                "Invalid pool type '{}'. Expected 'sanctum' or 'native'",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EpochStorageType {
    Memory,
    File,
}

impl EpochStorageType {
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "memory" => Ok(EpochStorageType::Memory),
            "file" => Ok(EpochStorageType::File),
            _ => Err(CrankerError::Config(format!(
                "Invalid epoch storage type '{}'. Expected 'memory' or 'file'",
                s
            ))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CrankerConfig {
    pub pool_type: PoolType,
    pub rpc_url: String,
    pub admin_private_key: String,
    pub pool_reserve_address: String,
    pub pool_address: Option<String>,
    pub crank_amount: u64,
    pub epoch_poll_interval: Duration,
    pub epoch_storage_type: EpochStorageType,
    pub epoch_state_file: String,
}

impl CrankerConfig {
    pub fn load() -> Result<Self> {
        let pool_type_str = env::var("POOL_TYPE")
            .map_err(|_| CrankerError::Config("POOL_TYPE not set".to_string()))?;
        let pool_type = PoolType::from_str(&pool_type_str)?;

        let rpc_url = env::var("RPC_URL")
            .map_err(|_| CrankerError::Config("RPC_URL not set".to_string()))?;

        let admin_private_key = env::var("ADMIN_PRIVATE_KEY")
            .map_err(|_| CrankerError::Config("ADMIN_PRIVATE_KEY not set".to_string()))?;

        let pool_reserve_address = env::var("POOL_RESERVE_ADDRESS")
            .map_err(|_| CrankerError::Config("POOL_RESERVE_ADDRESS not set".to_string()))?;

        let pool_address = env::var("POOL_ADDRESS").ok();

        let crank_amount_str = env::var("CRANK_AMOUNT")
            .map_err(|_| CrankerError::Config("CRANK_AMOUNT not set".to_string()))?;
        let crank_amount = crank_amount_str
            .parse::<u64>()
            .map_err(|e| CrankerError::Config(format!("Invalid CRANK_AMOUNT: {}", e)))?;

        let epoch_poll_interval_str = env::var("EPOCH_POLL_INTERVAL")
            .unwrap_or_else(|_| "5m".to_string());
        let epoch_poll_interval = parse_duration(&epoch_poll_interval_str)?;

        let epoch_storage_type_str = env::var("EPOCH_STORAGE_TYPE")
            .unwrap_or_else(|_| "memory".to_string());
        let epoch_storage_type = EpochStorageType::from_str(&epoch_storage_type_str)?;

        let epoch_state_file = env::var("EPOCH_STATE_FILE")
            .unwrap_or_else(|_| ".epoch_state".to_string());

        Ok(Self {
            pool_type,
            rpc_url,
            admin_private_key,
            pool_reserve_address,
            pool_address,
            crank_amount,
            epoch_poll_interval,
            epoch_storage_type,
            epoch_state_file,
        })
    }
}

pub fn parse_duration(s: &str) -> Result<Duration> {
    let s = s.trim();

    if s.is_empty() {
        return Err(CrankerError::Parse("Empty duration string".to_string()));
    }

    let (num_str, unit) = s.split_at(s.len() - 1);

    let number = num_str.parse::<u64>().map_err(|e| {
        CrankerError::Parse(format!("Invalid duration number '{}': {}", num_str, e))
    })?;

    let seconds = match unit {
        "s" => number,
        "m" => number * 60,
        "h" => number * 3600,
        "d" => number * 86400,
        _ => {
            return Err(CrankerError::Parse(format!(
                "Invalid duration unit '{}'. Use 's', 'm', 'h', or 'd'",
                unit
            )))
        }
    };

    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("1s").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_duration("30m").unwrap(), Duration::from_secs(1800));
        assert_eq!(parse_duration("1h").unwrap(), Duration::from_secs(3600));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86400));
        assert_eq!(parse_duration("12h").unwrap(), Duration::from_secs(43200));
    }

    #[test]
    fn test_pool_type_from_str() {
        assert_eq!(PoolType::from_str("sanctum").unwrap(), PoolType::Sanctum);
        assert_eq!(PoolType::from_str("Sanctum").unwrap(), PoolType::Sanctum);
        assert_eq!(PoolType::from_str("SANCTUM").unwrap(), PoolType::Sanctum);
        assert_eq!(PoolType::from_str("native").unwrap(), PoolType::Native);
        assert_eq!(PoolType::from_str("Native").unwrap(), PoolType::Native);
        assert!(PoolType::from_str("invalid").is_err());
    }
}
