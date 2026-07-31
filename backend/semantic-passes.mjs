// Semantic pass router — 4 passes on every bytecode output
// 🌊 telemetry → 🧠 policy → 🔒 sealing → 🔓 rights

import { createHash } from "crypto";

function blake3(data) {
  return createHash("sha256").update(typeof data === "string" ? data : JSON.stringify(data)).digest("hex");
}

// 🌊 Telemetry pass — count ops, track timing, emit metrics
export function telemetryPass(bytecodeTag) {
  const ops   = (bytecodeTag.payload || "").split(" ").filter(t => t && t !== "↩️");
  const nums  = ops.filter(t => t.startsWith("🔢")).length;
  const arith = ops.filter(t => ["➕","➖","✖️","➗","🤝"].includes(t)).length;
  return {
    pass: "telemetry",
    bytecodeHash: bytecodeTag.artifactHash,
    opCount: ops.length,
    numericPushes: nums,
    arithmeticOps: arith,
    stackDepthEst: Math.max(0, nums - arith),
    timestamp: new Date().toISOString(),
  };
}

// 🧠 Policy pass — Trust Deed v1.0 gate
export function policyPass(bytecodeTag, telemetry) {
  const payload = bytecodeTag.payload || "";
  const violations = [];

  if (!bytecodeTag.artifactHash) violations.push("missing artifact hash");
  if (!bytecodeTag.compiledFrom)  violations.push("missing compiledFrom CID");
  if (payload.trim() === "")      violations.push("empty bytecode payload");
  if (telemetry.opCount > 10000)  violations.push("bytecode exceeds safe op limit");

  const allowed = violations.length === 0;
  return {
    pass: "policy",
    engine: "trust-deed-v1.0",
    bytecodeHash: bytecodeTag.artifactHash,
    violations,
    decision: allowed ? "ALLOWED" : "DENIED",
    confidence: allowed ? 0.99 : 0.0,
  };
}

// 🔒 Sealing pass — WORM chain entry
export function sealingPass(bytecodeTag, executionTag, prevSeal) {
  const content = JSON.stringify({ bytecodeTag, executionTag });
  const hash    = blake3(content);
  const chain   = prevSeal ? blake3(prevSeal.seal + hash) : hash;
  return {
    pass: "sealing",
    seal: chain,
    bytecodeHash: bytecodeTag.artifactHash,
    resultHash: executionTag?.resultHash || null,
    previousSeal: prevSeal?.seal || null,
    algorithm: "sha256-chain",
    sealedAt: new Date().toISOString(),
    immutable: true,
  };
}

// 🔓 Rights pass — downgrade if policy denied or low confidence
export function rightsPass(policyResult, bobVerdict) {
  const score    = bobVerdict?.score ?? (policyResult.decision === "ALLOWED" ? 0.99 : 0.1);
  const downgrade = policyResult.decision === "DENIED" || score < 0.42;
  return {
    pass: "rights",
    score,
    capabilities: downgrade
      ? ["READ_ONLY"]
      : ["EXECUTE", "SEAL", "DELEGATE"],
    downgraded: downgrade,
    requiresHumanReview: downgrade,
    reason: downgrade
      ? `score ${score.toFixed(2)} < 0.42 or policy DENIED`
      : "full rights granted",
  };
}

// Run all 4 passes in sequence
export function runAllPasses(bytecodeTag, executionTag, prevSeal, bobVerdict) {
  const telemetry = telemetryPass(bytecodeTag);
  const policy    = policyPass(bytecodeTag, telemetry);
  const sealing   = sealingPass(bytecodeTag, executionTag, prevSeal);
  const rights    = rightsPass(policy, bobVerdict);
  return { telemetry, policy, sealing, rights };
}
