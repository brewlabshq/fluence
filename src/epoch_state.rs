use crate::config::EpochStorageType;
use crate::error::Result;
use std::fs;
use std::path::Path;

pub struct EpochState {
    storage_type: EpochStorageType,
    file_path: String,
    memory_epoch: Option<u64>,
}

impl EpochState {
    pub fn new(storage_type: EpochStorageType, file_path: String) -> Self {
        Self {
            storage_type,
            file_path,
            memory_epoch: None,
        }
    }

    pub fn load(&mut self) -> Result<Option<u64>> {
        match self.storage_type {
            EpochStorageType::Memory => Ok(self.memory_epoch),
            EpochStorageType::File => self.load_from_file(),
        }
    }

    pub fn save(&mut self, epoch: u64) -> Result<()> {
        match self.storage_type {
            EpochStorageType::Memory => {
                self.memory_epoch = Some(epoch);
                Ok(())
            }
            EpochStorageType::File => {
                self.memory_epoch = Some(epoch);
                self.save_to_file(epoch)
            }
        }
    }

    fn load_from_file(&mut self) -> Result<Option<u64>> {
        let path = Path::new(&self.file_path);
        if !path.exists() {
            tracing::debug!("Epoch state file does not exist, starting fresh");
            return Ok(None);
        }

        let content = fs::read_to_string(path)?;
        let epoch = content.trim().parse::<u64>().map_err(|e| {
            crate::error::CrankerError::Parse(format!(
                "Invalid epoch in state file '{}': {}",
                self.file_path, e
            ))
        })?;

        self.memory_epoch = Some(epoch);
        tracing::info!("Loaded last cranked epoch from file: {}", epoch);
        Ok(Some(epoch))
    }

    fn save_to_file(&self, epoch: u64) -> Result<()> {
        fs::write(&self.file_path, epoch.to_string())?;
        tracing::debug!("Saved epoch {} to state file", epoch);
        Ok(())
    }
}
