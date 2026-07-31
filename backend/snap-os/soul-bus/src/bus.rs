use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use craft_crypto::Dialect;
use soul_agent::AgentHandle;
use soulvm::BifrostBridge;

use crate::BusError;

#[derive(Clone)]
pub struct SoulBus {
    registry: Arc<Mutex<HashMap<u64, AgentHandle>>>,
}

impl SoulBus {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register(&self, soul_id: u64, handle: AgentHandle) -> Result<(), BusError> {
        let replaced = self
            .registry
            .lock()
            .map_err(|_| BusError::BusPoisoned)?
            .insert(soul_id, handle);
        if let Some(replaced) = replaced {
            replaced.shutdown();
        }
        Ok(())
    }

    pub fn spawn_soul(
        &self,
        soul_id: u64,
        bridge: Option<BifrostBridge>,
    ) -> Result<AgentHandle, BusError> {
        let handle = AgentHandle::spawn(soul_id, bridge)?;
        self.register(soul_id, handle.clone())?;
        Ok(handle)
    }

    pub fn call(&self, soul_id: u64, func_name: &str, args: Vec<i64>) -> Result<i64, BusError> {
        self.handle(soul_id)?
            .call(func_name, args)
            .map_err(Into::into)
    }

    pub fn load<D: Dialect>(
        &self,
        soul_id: u64,
        dialect: &D,
        source: &str,
    ) -> Result<String, BusError> {
        self.handle(soul_id)?
            .load_named(source, dialect)
            .map_err(Into::into)
    }

    pub fn broadcast(&self, func_name: &str, args: Vec<i64>) -> Vec<(u64, Result<i64, BusError>)> {
        let handles = match self.snapshot() {
            Ok(handles) => handles,
            Err(error) => return vec![(0, Err(error))],
        };

        handles
            .into_iter()
            .map(|(soul_id, handle)| {
                let result = handle.call(func_name, args.clone()).map_err(Into::into);
                (soul_id, result)
            })
            .collect()
    }

    pub fn despawn(&self, soul_id: u64) -> Result<(), BusError> {
        let handle = self
            .registry
            .lock()
            .map_err(|_| BusError::BusPoisoned)?
            .remove(&soul_id)
            .ok_or(BusError::UnknownSoul(soul_id))?;
        handle.shutdown();
        Ok(())
    }

    pub fn souls(&self) -> Vec<u64> {
        let Ok(registry) = self.registry.lock() else {
            return Vec::new();
        };
        let mut souls: Vec<_> = registry.keys().copied().collect();
        souls.sort_unstable();
        souls
    }

    pub fn soul_alive(&self, soul_id: u64) -> bool {
        self.handle(soul_id)
            .map(|handle| handle.is_alive())
            .unwrap_or(false)
    }

    fn handle(&self, soul_id: u64) -> Result<AgentHandle, BusError> {
        self.registry
            .lock()
            .map_err(|_| BusError::BusPoisoned)?
            .get(&soul_id)
            .cloned()
            .ok_or(BusError::UnknownSoul(soul_id))
    }

    fn snapshot(&self) -> Result<Vec<(u64, AgentHandle)>, BusError> {
        let registry = self.registry.lock().map_err(|_| BusError::BusPoisoned)?;
        let mut handles: Vec<_> = registry
            .iter()
            .map(|(soul_id, handle)| (*soul_id, handle.clone()))
            .collect();
        handles.sort_unstable_by_key(|(soul_id, _)| *soul_id);
        Ok(handles)
    }
}

impl Default for SoulBus {
    fn default() -> Self {
        Self::new()
    }
}
