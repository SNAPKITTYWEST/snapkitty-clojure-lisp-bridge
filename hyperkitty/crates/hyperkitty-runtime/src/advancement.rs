pub struct Runtime {
    pub current_state: crate::WorldState,
    pub tick_history: Vec<crate::Tick>,
    pub current_index: u64,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            current_state: crate::WorldState::new(),
            tick_history: vec![],
            current_index: 0,
        }
    }

    pub fn advance_tick(&mut self, action: crate::Action, _proof: crate::ProofObligation) -> hyperkitty_core::Result<()> {
        let sigma_in = self.current_state.clone();
        let mut sigma_out = sigma_in.clone();
        action.apply(&mut sigma_out)?;
        
        let tick = crate::Tick::new(self.current_index, sigma_in, action, sigma_out.clone());
        self.tick_history.push(tick);
        self.current_state = sigma_out;
        self.current_index += 1;
        Ok(())
    }

    pub fn verify_all_ticks(&self) -> bool {
        self.tick_history.iter().all(|t| t.verify())
    }
}

impl Default for Runtime {
    fn default() -> Self { Self::new() }
}
