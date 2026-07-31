/// soul-narrator — turns machine events into plain human language.
/// Rule-based, Sprint 1. LLM layer wires in Sprint 2.

#[derive(Debug, Clone, PartialEq)]
pub enum MachineEvent {
    SoulSpawned       { soul_id: u64 },
    SoulDespawned     { soul_id: u64 },
    FuncLoaded        { soul_id: u64, func_name: String },
    FuncCalled        { soul_id: u64, func_name: String, result: i64 },
    ChainSealed       { cid: String, event_type: String },
    FluxConsultation  { soul_id: u64, instrument: String, steps: usize },
    HealthSignal      (String),
}

pub struct Narrator;

impl Narrator {
    pub fn narrate(event: &MachineEvent) -> String {
        match event {
            MachineEvent::SoulSpawned { soul_id } =>
                format!("Soul {soul_id} has come online and is ready."),

            MachineEvent::SoulDespawned { soul_id } =>
                format!("Soul {soul_id} has gone offline."),

            MachineEvent::FuncLoaded { soul_id, func_name } =>
                format!("Soul {soul_id} loaded a new program called {func_name}."),

            MachineEvent::FuncCalled { soul_id, func_name: _, result } =>
                format!("Soul {soul_id} ran a program and got {result}."),

            MachineEvent::ChainSealed { cid: _, event_type } =>
                format!("A new {event_type} was sealed to the permanent record."),

            MachineEvent::FluxConsultation { soul_id, instrument, steps } =>
                format!(
                    "Soul {soul_id} received a financial plan with {steps} step{} to {instrument}.",
                    if *steps == 1 { "" } else { "s" },
                    instrument = instrument.to_lowercase(),
                ),

            MachineEvent::HealthSignal(msg) =>
                format!("System health: {msg}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn narrate_each_variant() {
        assert_eq!(
            Narrator::narrate(&MachineEvent::SoulSpawned { soul_id: 1 }),
            "Soul 1 has come online and is ready."
        );
        assert_eq!(
            Narrator::narrate(&MachineEvent::SoulDespawned { soul_id: 2 }),
            "Soul 2 has gone offline."
        );
        assert_eq!(
            Narrator::narrate(&MachineEvent::FuncLoaded {
                soul_id: 3, func_name: "my_fn".into()
            }),
            "Soul 3 loaded a new program called my_fn."
        );
        assert_eq!(
            Narrator::narrate(&MachineEvent::FuncCalled {
                soul_id: 1, func_name: "emoji_fn_abc".into(), result: 42
            }),
            "Soul 1 ran a program and got 42."
        );
        assert!(
            Narrator::narrate(&MachineEvent::HealthSignal("all good".into()))
                .contains("all good")
        );
    }

    #[test]
    fn narrate_flux_consultation_mentions_instrument() {
        let s = Narrator::narrate(&MachineEvent::FluxConsultation {
            soul_id: 5,
            instrument: "Basic Bank Account".into(),
            steps: 4,
        });
        assert!(s.contains("Soul 5"), "missing soul id: {s}");
        assert!(s.contains("4 steps"), "missing steps: {s}");
        assert!(s.contains("basic bank account"), "missing instrument: {s}");
    }

    #[test]
    fn narrate_chain_seal_mentions_record() {
        let s = Narrator::narrate(&MachineEvent::ChainSealed {
            cid: "abc123".into(),
            event_type: "JitCompile".into(),
        });
        assert!(s.contains("permanent record"), "missing record: {s}");
        assert!(s.contains("JitCompile"), "missing event type: {s}");
    }
}
