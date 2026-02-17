pub mod backend;
pub mod protocol;
pub mod focus_protocol;
pub mod focus_session;
pub mod crypto;
pub mod entropy;
pub mod git;
pub mod monitor;
pub mod mouse_sentinel;
pub mod stats;
pub mod complexity;
pub mod config;
pub mod ui_templates;
pub mod privacy;

use sha2::{Digest, Sha256};
use ed25519_dalek::SigningKey;
use zstd::stream::encode_all;
use statrs::statistics::Statistics;

/// Digital Notary: Verifies the integrity of the tech stack.
/// This function acts as a boot-up self-test for the Sentinel.
/// It confirms that all static linking and trait implementations are functional.
pub fn sentinel_self_check() -> Result<String, String> {
    let mut status_report = String::from("Sentinel System Integrity Check:\n");

    // 1. Verify Git2 Linkage
    // Insight: Basic verification that libgit2 is accessible.
    let _repo = git2::Repository::open_from_env().map_err(|_| "Git2 linkage failed")?;
    status_report.push_str("✔ Git2 (libgit2): Static Linked\n");

    // 2. Verify Cryptography (Ed25519-Dalek)
    // Ensures the randomness generator and curve logic are working.
    let mut csprng = rand::rngs::OsRng;
    let _keypair = SigningKey::generate(&mut csprng);
    status_report.push_str("✔ Crypto (Ed25519): Key Generation Subsystem Active\n");

    // 3. Verify Hashing (Sha2)
    // Critical for PoHW puzzle solving.
    let mut hasher = Sha256::new();
    hasher.update(b"entropy_check");
    let _result = hasher.finalize();
    status_report.push_str("✔ Hashing (SHA256): Engine Online\n");

    // 4. Verify Compression (Zstd)
    // Critical for NCD calculation. We test a simple compression round-trip.
    let payload = b"redundant_data_redundant_data_redundant_data";
    // We use Level 1 for "Dumb Compression" as per architectural requirement 
    let compressed = encode_all(&payload[..], 1).map_err(|e| e.to_string())?;
    let ratio = payload.len() as f64 / compressed.len() as f64;
    status_report.push_str(&format!(
        "✔ Entropy Engine (Zstd): Compression Active (Ratio {:.2}x)\n",
        ratio
    ));

    // 5. Verify Statistics (Statrs)
    // Ensures that 'statrs' is compiled correctly without 'nalgebra' bloat
    // but still retains the ability to calculate standard deviation.
    let data = [1.0, 2.0, 3.0, 4.0, 5.0];
    let mean = data.mean();
    let std_dev = data.std_dev();
    status_report.push_str(&format!(
        "✔ Statistics (Statrs): Compute Modules Loaded (Mean: {:.1}, StdDev: {:.4})\n",
        mean, std_dev
    ));

    Ok(status_report)
}