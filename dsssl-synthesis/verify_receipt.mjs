#!/usr/bin/env node
import { createHash } from 'crypto';
import TweetNaCl from 'tweetnacl';
import fs from 'fs';

const receipt = JSON.parse(fs.readFileSync('./synthesis_receipt.json', 'utf8'));

console.log('╔════════════════════════════════════════════════════════════════╗');
console.log('║            INDEPENDENT RECEIPT VERIFICATION                   ║');
console.log('╚════════════════════════════════════════════════════════════════╝\n');

// Verify Ed25519 signature
const publicKeyHex = receipt.cryptographic_proof.public_key;
const signatureHex = receipt.cryptographic_proof.signature;
const message = receipt.cryptographic_proof.message;

const publicKeyBuf = Buffer.from(publicKeyHex, 'hex');
const signatureBuf = Buffer.from(signatureHex, 'hex');
const messageBuf = Buffer.from(message, 'utf8');

const isValidSig = TweetNaCl.sign.detached.verify(messageBuf, signatureBuf, publicKeyBuf);

console.log('[SIGNATURE VERIFICATION]');
console.log(`  Public key: ${publicKeyHex.substring(0, 32)}...`);
console.log(`  Message: ${message.substring(0, 32)}...`);
console.log(`  Signature: ${signatureHex.substring(0, 64)}...`);
console.log(`  Result: ${isValidSig ? '✓ VALID' : '✗ INVALID'}`);

// Verify hash reproducibility
const receiptContent = {
  version: receipt.version,
  timestamp: receipt.timestamp,
  algorithm: receipt.algorithm,
  synthesis: receipt.synthesis,
  hashes: {
    ast: receipt.hashes.ast,
    code: receipt.hashes.code,
    lean4_cert: receipt.hashes.lean4_cert
  }
};

const recomputedHash = createHash('sha3-256')
  .update(JSON.stringify(receiptContent))
  .digest('hex');

console.log('\n[HASH REPRODUCIBILITY]');
console.log(`  Expected: ${receipt.hashes.receipt_content}`);
console.log(`  Computed: ${recomputedHash}`);
console.log(`  Match: ${recomputedHash === receipt.hashes.receipt_content ? '✓ YES' : '✗ NO'}`);

// Verify Lean certificate hash
const leanCert = fs.readFileSync('./append_certificate.lean', 'utf8');
const computedLeanHash = createHash('sha3-256')
  .update(leanCert)
  .digest('hex');

console.log('\n[LEAN4 CERTIFICATE HASH]');
console.log(`  Expected: ${receipt.hashes.lean4_cert}`);
console.log(`  Computed: ${computedLeanHash}`);
console.log(`  Match: ${computedLeanHash === receipt.hashes.lean4_cert ? '✓ YES' : '✗ NO'}`);

// Final verdict
const allValid = isValidSig && (recomputedHash === receipt.hashes.receipt_content) && (computedLeanHash === receipt.hashes.lean4_cert);

console.log('\n╔════════════════════════════════════════════════════════════════╗');
if (allValid) {
  console.log('║                    ✓ ALL VERIFICATIONS PASSED                   ║');
} else {
  console.log('║                    ✗ VERIFICATION FAILED                        ║');
}
console.log('╚════════════════════════════════════════════════════════════════╝\n');

process.exit(allValid ? 0 : 1);
