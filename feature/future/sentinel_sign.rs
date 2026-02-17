//! Sentinel Signatures
//!
//! Cryptographic signing and verification for SentinelReport integrity.
//!
//! RESPONSIBILITIES:
//! - Sign a content hash.
//! - Verify signatures against a content hash.
//!
//! NON-RESPONSIBILITIES:
//! - No hashing.
//! - No PoW.
//! - No IO.
//! - No key management.
//! - No policy.
//!
//! This module is pure cryptographic plumbing.

use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error)]
pub enum SignError {
    #[error("signature verification failed")]
    InvalidSignature,

    #[error("unknown signature scheme")]
    UnknownScheme,
}

/// Supported signature schemes.
///
/// This enum is intentionally minimal and explicit.
/// Extension requires an additive change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureScheme {
    Ed25519,
}

impl SignatureScheme {
    pub fn as_str(&self) -> &'static str {
        match self {
            SignatureScheme::Ed25519 => "ed25519",
        }
    }
}

/// Sign a content hash.
///
/// - `content_hash` must already be canonical and stable.
/// - `sign_fn` is injected to avoid key ownership here.
pub fn sign_content_hash<F>(
    scheme: SignatureScheme,
    content_hash: &[u8],
    sign_fn: F,
) -> Result<(String, String), SignError>
where
    F: Fn(&[u8]) -> Vec<u8>,
{
    let signature = match scheme {
        SignatureScheme::Ed25519 => sign_fn(content_hash),
    };

    Ok((
        scheme.as_str().to_string(),
        hex::encode(signature),
    ))
}

/// Verify a content hash signature.
///
/// - `verify_fn` is injected and owns key semantics.
pub fn verify_signature<F>(
    scheme: &str,
    content_hash: &[u8],
    signature_hex: &str,
    verify_fn: F,
) -> Result<(), SignError>
where
    F: Fn(&[u8], &[u8]) -> bool,
{
    let signature =
        hex::decode(signature_hex).map_err(|_| SignError::InvalidSignature)?;

    match scheme {
        "ed25519" => {
            if verify_fn(content_hash, &signature) {
                Ok(())
            } else {
                Err(SignError::InvalidSignature)
            }
        }
        _ => Err(SignError::UnknownScheme),
    }
}

/// Verify all signatures in an integrity envelope.
///
/// Fails fast on first invalid signature.
pub fn verify_all_signatures<F>(
    content_hash: &[u8],
    signatures: &BTreeMap<String, String>,
    mut verifier: F,
) -> Result<(), SignError>
where
    F: FnMut(&str, &[u8], &[u8]) -> bool,
{
    for (scheme, sig_hex) in signatures {
        let sig = hex::decode(sig_hex).map_err(|_| SignError::InvalidSignature)?;

        if !verifier(scheme, content_hash, &sig) {
            return Err(SignError::InvalidSignature);
        }
    }

    Ok(())
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_and_verify_roundtrip() {
        let content_hash = b"canonical-hash";

        // Fake signer (identity)
        let (scheme, sig) = sign_content_hash(
            SignatureScheme::Ed25519,
            content_hash,
            |data| data.to_vec(),
        )
        .unwrap();

        // Fake verifier (identity)
        let result = verify_signature(
            &scheme,
            content_hash,
            &sig,
            |data, sig| data == sig,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_invalid_signature() {
        let content_hash = b"canonical-hash";
        let bad_sig = hex::encode(b"nope");

        let result = verify_signature(
            "ed25519",
            content_hash,
            &bad_sig,
            |data, sig| data == sig,
        );

        assert!(result.is_err());
    }
}
