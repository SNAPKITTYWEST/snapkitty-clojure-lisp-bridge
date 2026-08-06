pub struct InodeFlags {
    pub base_dir: String,
}

impl InodeFlags {
    pub fn new(base_dir: String) -> Self {
        Self { base_dir }
    }

    pub fn set_flag(&self, _name: &str) -> hyperkitty_core::Result<()> {
        Ok(())
    }

    pub fn clear_flag(&self, _name: &str) -> hyperkitty_core::Result<()> {
        Ok(())
    }

    pub fn get_flag(&self, _name: &str) -> bool {
        false
    }
}
