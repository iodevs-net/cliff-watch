//! Sentinel Signer — cryptographic attestation
//!
//! RESPONSIBILITIES:
//! - Sign content hashes.
//! - Verify signatures.
//!
//! NON-RESPONSIBILITIES:
//! - No hashing.
//! - No key storage.
//! - No IO.

use ed25519_dalek::{Signature, SigningKey, VerifyingKey, Signer, Verifier};

#[derive(Debug, thiserror::Error)]
pub enum SignError {
    #[error("signature verification failed")]
    VerificationFailed,
}

/// Sign a content hash (hex string).
pub fn sign_hash(
    signing_key: &SigningKey,
    content_hash_hex: &str,
) -> String {
    let sig: Signature = signing_key.sign(content_hash_hex.as_bytes());
    hex::encode(sig.to_bytes())
}

/// Verify a signature against a content hash.
pub fn verify_signature(
    verifying_key: &VerifyingKey,
    content_hash_hex: &str,
    signature_hex: &str,
) -> Result<(), SignError> {
    let sig_bytes = hex::decode(signature_hex).map_err(|_| SignError::VerificationFailed)?;
    let sig = Signature::from_bytes(&sig_bytes.try_into().map_err(|_| SignError::VerificationFailed)?);

    verifying_key
        .verify(content_hash_hex.as_bytes(), &sig)
        .map_err(|_| SignError::VerificationFailed)
}
