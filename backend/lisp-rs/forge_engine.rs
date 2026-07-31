//! FORGE Generation Engine — Autonomous Environmental Evolution
//!
//! FORGE does not generate terrain. FORGE generates:
//!   infrastructure logic, settlement patterns, ecosystem balance,
//!   social topology, conflict vectors, economic pressure, architectural identity.
//!
//! When a player says "build a trading post that charges 2% tax" —
//! FORGE constructs the shard, authors the object DNA, writes the behavior rules,
//! patches the economy, and seals the entire act to the WORM chain.
//! No developer wrote this map. FORGE built it. It is now permanent history.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use super::object_dna::{ObjectDna, ObjectKind, BehaviorRule};
use super::world_shard::{WorldShard, ShardKind};

/// A FORGE generation request — what the world needs built
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeRequest {
    pub id:          String,
    pub intent:      String,      // raw player/agent intent
    pub parsed_kind: GenerationKind,
    pub district:    String,
    pub constraints: Vec<String>, // economic, physical, social constraints
    pub requester:   String,      // player ID or agent name
    pub seal:        String,
    pub ts:          u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GenerationKind {
    Structure,       // buildings, walls, roads
    Economy,         // markets, vaults, banks, tax systems
    Ecosystem,       // forests, rivers, fauna populations
    Social,          // community spaces, governance halls
    Infrastructure,  // roads, bridges, portals between shards
    Conflict,        // arenas, faction territories, contested zones
    Event,           // temporary world events — festivals, disasters
}

/// A FORGE generation result — what was built
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeOutput {
    pub request_id:   String,
    pub shard:        WorldShard,
    pub objects:      Vec<ObjectDna>,
    pub behavior_script: String,       // Lua script for runtime behavior
    pub economic_rules:  Vec<EconomicRule>,
    pub seal:         String,
    pub worm_id:      String,
    pub build_ms:     u64,
    pub ts:           u64,
}

/// An economic rule authored by FORGE for a shard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicRule {
    pub id:          String,
    pub kind:        EconomicRuleKind,
    pub rate:        f64,     // tax rate, interest rate, exchange rate
    pub condition:   String,  // when this rule applies
    pub beneficiary: String,  // who receives the value
    pub seal:        String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EconomicRuleKind {
    Tax,
    Tariff,
    Subsidy,
    InterestRate,
    ExchangeRate,
    ResourceCap,
}

/// Settlement emergence pattern — how FORGE decides what to build next
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementPressure {
    pub population_density: f64,    // drives housing and social infra
    pub trade_volume:        f64,    // drives markets and roads
    pub conflict_index:      f64,    // drives arenas and fortifications
    pub resource_scarcity:   f64,    // drives resource extraction structures
    pub cultural_momentum:   f64,    // drives galleries, schools, temples
}

impl SettlementPressure {
    /// Determine what FORGE should build next based on systemic pressure
    pub fn highest_priority(&self) -> GenerationKind {
        let pressures = [
            (self.population_density, GenerationKind::Social),
            (self.trade_volume,       GenerationKind::Economy),
            (self.conflict_index,     GenerationKind::Conflict),
            (self.resource_scarcity,  GenerationKind::Infrastructure),
            (self.cultural_momentum,  GenerationKind::Structure),
        ];
        pressures.iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .map(|(_, k)| k.clone())
            .unwrap_or(GenerationKind::Structure)
    }
}

impl ForgeRequest {
    pub fn new(
        intent: impl Into<String>,
        parsed_kind: GenerationKind,
        district: impl Into<String>,
        requester: impl Into<String>,
    ) -> Self {
        let ts        = Self::now();
        let intent    = intent.into();
        let district  = district.into();
        let requester = requester.into();

        let mut h = Sha256::new();
        h.update(format!("FORGE-REQ:{:?}:{}:{}:{}", parsed_kind, district, requester, ts).as_bytes());
        let seal = hex::encode(h.finalize());
        let id   = format!("FREQ-{}", &seal[..12].to_uppercase());

        Self { id, intent, parsed_kind, district, constraints: vec![], requester, seal, ts }
    }

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}

/// The FORGE engine — parses intent, constructs shards and objects
pub struct ForgeEngine {
    pub generations: Vec<ForgeOutput>,
}

impl ForgeEngine {
    pub fn new() -> Self { Self { generations: vec![] } }

    /// Parse raw player/agent intent into a structured request
    pub fn parse_intent(
        &self,
        raw_intent: &str,
        district: &str,
        requester: &str,
    ) -> ForgeRequest {
        let kind = Self::classify_intent(raw_intent);
        ForgeRequest::new(raw_intent, kind, district, requester)
    }

    /// Execute a FORGE request — build the shard, objects, and rules
    pub fn execute(&mut self, req: ForgeRequest) -> ForgeOutput {
        let t0 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let (shard, objects, econ_rules, script) = match req.parsed_kind {
            GenerationKind::Economy   => self.build_economy(&req),
            GenerationKind::Structure => self.build_structure(&req),
            GenerationKind::Social    => self.build_social(&req),
            GenerationKind::Conflict  => self.build_conflict(&req),
            GenerationKind::Ecosystem => self.build_ecosystem(&req),
            GenerationKind::Infrastructure => self.build_infrastructure(&req),
            GenerationKind::Event     => self.build_event(&req),
        };

        let ts      = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let build_ms = ts - t0;

        let mut h = Sha256::new();
        h.update(format!("FORGE-OUT:{}:{}:{}", req.id, shard.id, ts).as_bytes());
        let seal   = hex::encode(h.finalize());
        let worm_id = format!("WORM-FORGE-{}", &seal[..16].to_uppercase());

        let output = ForgeOutput {
            request_id: req.id, shard, objects, behavior_script: script,
            economic_rules: econ_rules, seal, worm_id, build_ms, ts,
        };

        self.generations.push(output.clone());
        output
    }

    // ── Generation Routines ──────────────────────────────────────────────

    fn build_economy(&self, req: &ForgeRequest) -> (WorldShard, Vec<ObjectDna>, Vec<EconomicRule>, String) {
        // Parse tax rate from intent if present
        let tax_rate = Self::extract_rate(&req.intent, "tax").unwrap_or(0.02);

        let mut shard = WorldShard::spawn(
            ShardKind::TradingPost,
            Self::name_from_intent(&req.intent, "Exchange"),
            &req.district, 200,
            format!("FORGE built by: {}", req.requester),
        );

        // Object: market structure
        let mut market = ObjectDna::new(ObjectKind::Economic, "Market Hall");
        market.attributes.insert("trade_volume".into(), 0.0);
        market.attributes.insert("tax_rate".into(), tax_rate);
        market.forge_rewrite(
            "on_trade",
            &format!("price > 0"),
            &format!("apply_tax({tax_rate}); seal_ledger(); emit_to_vault()"),
        );

        // Object: vault
        let mut vault = ObjectDna::new(ObjectKind::Economic, "Tax Vault");
        vault.attributes.insert("balance".into(), 0.0);

        // Economic rule
        let mut h = Sha256::new();
        h.update(format!("TAX:{}:{}", tax_rate, req.ts).as_bytes());
        let rule_seal = hex::encode(h.finalize());
        let econ_rule = EconomicRule {
            id:          format!("ERULE-{}", &rule_seal[..10].to_uppercase()),
            kind:        EconomicRuleKind::Tax,
            rate:        tax_rate,
            condition:   "every_trade".into(),
            beneficiary: req.requester.clone(),
            seal:        rule_seal,
        };

        shard.forge_patch(format!(
            "-- FORGE generated economy script\nTAX_RATE = {tax_rate}\n\
             function on_trade(item, price)\n  tax = price * TAX_RATE\n\
             ledger:seal(item, price, tax)\n  vault:deposit(tax)\nend"
        ));

        let script = format!(
            "-- Sovereign trading post: {}\n-- Tax: {}%\n-- WORM-sealed every trade",
            req.intent, tax_rate * 100.0
        );

        (shard, vec![market, vault], vec![econ_rule], script)
    }

    fn build_structure(&self, req: &ForgeRequest) -> (WorldShard, Vec<ObjectDna>, Vec<EconomicRule>, String) {
        let mut shard = WorldShard::spawn(
            ShardKind::BuildZone,
            Self::name_from_intent(&req.intent, "Structure"),
            &req.district, 50,
            format!("FORGE built by: {}", req.requester),
        );
        let building = ObjectDna::new(ObjectKind::Structure, &req.intent);
        let script = format!("-- Structure: {}\n-- All state sealed to WORM", req.intent);
        shard.forge_patch(&script);
        (shard, vec![building], vec![], script)
    }

    fn build_social(&self, req: &ForgeRequest) -> (WorldShard, Vec<ObjectDna>, Vec<EconomicRule>, String) {
        let mut shard = WorldShard::spawn(ShardKind::SocialHub,
            Self::name_from_intent(&req.intent, "Hub"), &req.district, 150,
            format!("FORGE: {}", req.intent));
        let hall = ObjectDna::new(ObjectKind::Structure, "Community Hall");
        let script = "-- Social hub: NPC memory events sealed here\n-- Governance voting enabled".into();
        shard.forge_patch(script);
        (shard, vec![hall], vec![], "-- community hub script".into())
    }

    fn build_conflict(&self, req: &ForgeRequest) -> (WorldShard, Vec<ObjectDna>, Vec<EconomicRule>, String) {
        let mut shard = WorldShard::spawn(ShardKind::CombatArena,
            Self::name_from_intent(&req.intent, "Arena"), &req.district, 40,
            format!("FORGE: {}", req.intent));
        let arena = ObjectDna::new(ObjectKind::Structure, "Combat Arena");
        let script = "-- Combat arena: every kill sealed to weapon DNA + player ledger".into();
        shard.forge_patch(script);
        (shard, vec![arena], vec![], "-- combat script".into())
    }

    fn build_ecosystem(&self, req: &ForgeRequest) -> (WorldShard, Vec<ObjectDna>, Vec<EconomicRule>, String) {
        let mut shard = WorldShard::spawn(ShardKind::Wilderness,
            Self::name_from_intent(&req.intent, "Wilderness"), &req.district, 500,
            format!("FORGE: {}", req.intent));
        let terrain = ObjectDna::new(ObjectKind::NaturalFeature, "Procedural Terrain");
        let script = "-- Ecosystem: resource nodes, fauna population, weather sealed per tick".into();
        shard.forge_patch(script);
        (shard, vec![terrain], vec![], "-- ecosystem script".into())
    }

    fn build_infrastructure(&self, req: &ForgeRequest) -> (WorldShard, Vec<ObjectDna>, Vec<EconomicRule>, String) {
        let mut shard = WorldShard::spawn(ShardKind::Portal,
            Self::name_from_intent(&req.intent, "Portal"), &req.district, 1000,
            format!("FORGE: {}", req.intent));
        let portal = ObjectDna::new(ObjectKind::Portal, "Shard Gate");
        let script = "-- Portal: cross-shard travel point, toll optional, usage sealed to ledger".into();
        shard.forge_patch(script);
        (shard, vec![portal], vec![], "-- infrastructure script".into())
    }

    fn build_event(&self, req: &ForgeRequest) -> (WorldShard, Vec<ObjectDna>, Vec<EconomicRule>, String) {
        let mut shard = WorldShard::spawn(ShardKind::EventSpace,
            Self::name_from_intent(&req.intent, "Event"), &req.district, 300,
            format!("FORGE event: {}", req.intent));
        let stage = ObjectDna::new(ObjectKind::Structure, "Event Stage");
        let script = "-- Temporary event shard: auto-tears down when players leave".into();
        shard.forge_patch(script);
        (shard, vec![stage], vec![], "-- event script".into())
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    fn classify_intent(intent: &str) -> GenerationKind {
        let i = intent.to_lowercase();
        if i.contains("tax") || i.contains("market") || i.contains("trade") || i.contains("economy") || i.contains("bank") {
            GenerationKind::Economy
        } else if i.contains("arena") || i.contains("combat") || i.contains("fight") || i.contains("war") {
            GenerationKind::Conflict
        } else if i.contains("forest") || i.contains("river") || i.contains("nature") || i.contains("farm") {
            GenerationKind::Ecosystem
        } else if i.contains("road") || i.contains("bridge") || i.contains("portal") || i.contains("gate") {
            GenerationKind::Infrastructure
        } else if i.contains("festival") || i.contains("event") || i.contains("celebration") {
            GenerationKind::Event
        } else if i.contains("hall") || i.contains("community") || i.contains("gather") || i.contains("social") {
            GenerationKind::Social
        } else {
            GenerationKind::Structure
        }
    }

    fn extract_rate(intent: &str, keyword: &str) -> Option<f64> {
        let pattern = format!("{}%", keyword);
        if let Some(pos) = intent.to_lowercase().find(&pattern) {
            let before = &intent[..pos];
            let num_start = before.rfind(|c: char| !c.is_ascii_digit() && c != '.').map(|i| i + 1).unwrap_or(0);
            if let Ok(rate) = before[num_start..].parse::<f64>() {
                return Some(rate / 100.0);
            }
        }
        // Also check "N% tax" pattern
        if intent.to_lowercase().contains(keyword) {
            // scan for any number near the keyword
            for token in intent.split_whitespace() {
                let t = token.trim_end_matches('%');
                if let Ok(v) = t.parse::<f64>() {
                    if v > 0.0 && v <= 100.0 {
                        return Some(v / 100.0);
                    }
                }
            }
        }
        None
    }

    fn name_from_intent(intent: &str, fallback: &str) -> String {
        // Use first 3-4 significant words of intent as the name
        let words: Vec<&str> = intent.split_whitespace()
            .filter(|w| w.len() > 3)
            .take(3)
            .collect();
        if words.is_empty() { fallback.to_string() } else { words.join(" ") }
    }
}

impl Default for ForgeEngine {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_economy_intent() {
        let engine = ForgeEngine::new();
        let req = engine.parse_intent(
            "Build a trading post that charges 2% tax",
            "Downtown", "PLAYER-001",
        );
        assert_eq!(req.parsed_kind, GenerationKind::Economy);
    }

    #[test]
    fn test_forge_builds_economy_shard() {
        let mut engine = ForgeEngine::new();
        let req = engine.parse_intent("Build a trading post that charges 2% tax", "Downtown", "PLAYER");
        let output = engine.execute(req);

        assert!(!output.shard.id.is_empty());
        assert!(!output.objects.is_empty());
        assert!(!output.economic_rules.is_empty());
        assert_eq!(output.economic_rules[0].kind, EconomicRuleKind::Tax);
        assert!((output.economic_rules[0].rate - 0.02).abs() < 0.001);
        assert!(!output.worm_id.is_empty());
        assert!(output.worm_id.starts_with("WORM-FORGE-"));
    }

    #[test]
    fn test_forge_conflict_arena() {
        let mut engine = ForgeEngine::new();
        let req = engine.parse_intent("create a combat arena for player tournaments", "Underground", "NEXUS");
        let output = engine.execute(req);
        assert_eq!(output.shard.kind, ShardKind::CombatArena);
    }

    #[test]
    fn test_settlement_pressure_priorities() {
        let high_trade = SettlementPressure {
            population_density: 0.3,
            trade_volume:       0.9,
            conflict_index:     0.2,
            resource_scarcity:  0.1,
            cultural_momentum:  0.4,
        };
        assert_eq!(high_trade.highest_priority(), GenerationKind::Economy);

        let high_conflict = SettlementPressure {
            population_density: 0.2, trade_volume: 0.3,
            conflict_index: 0.95, resource_scarcity: 0.1, cultural_momentum: 0.2,
        };
        assert_eq!(high_conflict.highest_priority(), GenerationKind::Conflict);
    }
}
