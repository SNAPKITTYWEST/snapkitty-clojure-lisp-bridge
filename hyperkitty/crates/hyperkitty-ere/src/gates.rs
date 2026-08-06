//! ERE Five-gate protocol

use crate::Result;
use hyperkitty_core::Error;

pub struct EreGates {
    pub p1_no_secrets: bool,
    pub p2_no_eval: bool,
    pub p3_loop_safety: bool,
    pub p4_no_telemetry: bool,
}

impl EreGates {
    pub fn new() -> Self {
        EreGates {
            p1_no_secrets: true,
            p2_no_eval: true,
            p3_loop_safety: true,
            p4_no_telemetry: true,
        }
    }

    pub fn check_p1_no_secrets(content: &str) -> bool {
        // Check for credentials/keys
        !content.contains("password")
            && !content.contains("token")
            && !content.contains("secret")
    }

    pub fn check_p2_no_eval(content: &str) -> bool {
        // Check for eval/exec
        !content.contains("eval") && !content.contains("exec")
    }

    pub fn check_p3_loop_safety(_content: &str) -> bool {
        // Stub: always safe
        true
    }

    pub fn check_p4_no_telemetry(content: &str) -> bool {
        // Check for telemetry/beacons
        !content.contains("beacon") && !content.contains("telemetry")
    }

    pub fn all_pass(&self) -> bool {
        self.p1_no_secrets && self.p2_no_eval && self.p3_loop_safety && self.p4_no_telemetry
    }
}

impl Default for EreGates {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p1_detects_secrets() {
        assert!(!EreGates::check_p1_no_secrets("password=admin"));
        assert!(EreGates::check_p1_no_secrets("normal content"));
    }

    #[test]
    fn p2_detects_eval() {
        assert!(!EreGates::check_p2_no_eval("eval(code)"));
        assert!(EreGates::check_p2_no_eval("normal content"));
    }
}
