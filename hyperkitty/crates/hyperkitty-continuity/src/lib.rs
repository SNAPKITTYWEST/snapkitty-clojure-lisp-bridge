pub mod environment;
pub mod seed_chain;
pub mod inode;
pub mod shared_memory;

pub use environment::EnvironmentBitmask;
pub use seed_chain::SeedChain;
pub use shared_memory::SharedMemory;

pub trait ContinuityEngine {
    fn init(&mut self) -> hyperkitty_core::Result<()>;
    fn store(&mut self, key: &str, value: &[u8]) -> hyperkitty_core::Result<()>;
    fn load(&self, key: &str) -> hyperkitty_core::Result<Option<Vec<u8>>>;
    fn checkpoint(&mut self) -> hyperkitty_core::Result<hyperkitty_core::Hash>;
    fn verify_integrity(&self) -> hyperkitty_core::Result<bool>;
}
