#[derive(Debug, Clone)]
pub enum Action {
    SetFact(String, String),
    RemoveFact(String),
    Batch(Vec<Action>),
}

impl Action {
    pub fn apply(&self, world: &mut crate::WorldState) -> hyperkitty_core::Result<()> {
        match self {
            Action::SetFact(k, v) => { world.set(k.clone(), v.clone()); Ok(()) }
            Action::RemoveFact(k) => { world.facts.remove(k); Ok(()) }
            Action::Batch(acts) => { for a in acts { a.apply(world)?; } Ok(()) }
        }
    }
}
