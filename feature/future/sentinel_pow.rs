//! Adaptive Proof-of-Work (PoW)
//!
//! Governance friction, not mining.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Difficulty(pub u8);

#[derive(Debug, Clone)]
pub struct PowChallenge {
    pub seed: [u8; 32],
    pub difficulty: Difficulty,
}

#[derive(Debug, Clone)]
pub struct PowSolution {
    pub nonce: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum PowError {
    #[error("Invalid proof of work")]
    Invalid,
}

/// Create deterministic PoW challenge.
pub fn create_challenge(
    seed: [u8; 32],
    entropy_score: f64,
) -> PowChallenge {
    PowChallenge {
        seed,
        difficulty: difficulty_from_entropy(entropy_score),
    }
}

/// Verify a PoW solution.
pub fn verify_solution(
    challenge: &PowChallenge,
    solution: &PowSolution,
) -> Result<(), PowError> {
    let hash = compute_hash(&challenge.seed, solution.nonce);

    if !meets_difficulty(&hash, challenge.difficulty) {
        return Err(PowError::Invalid);
    }

    Ok(())
}

fn compute_hash(seed: &[u8; 32], nonce: u64) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(seed);
    hasher.update(nonce.to_le_bytes());
    hasher.finalize().into()
}

fn meets_difficulty(hash: &[u8; 32], difficulty: Difficulty) -> bool {
    let mut remaining = difficulty.0;

    for byte in hash {
        let leading = byte.leading_zeros() as u8;

        if leading >= remaining {
            return true;
        }

        if leading < 8 {
            return false;
        }

        remaining -= 8;
    }

    false
}

/// Policy layer: entropy → difficulty
pub fn difficulty_from_entropy(entropy: f64) -> Difficulty {
    Difficulty(map_entropy_to_difficulty(entropy))
}

fn map_entropy_to_difficulty(entropy: f64) -> u8 {
    let e = entropy.clamp(0.0, 1.0);

    match e {
        e if e < 0.2 => 0,
        e if e < 0.4 => 4,
        e if e < 0.6 => 8,
        e if e < 0.8 => 12,
        _ => 16,
    }
}
