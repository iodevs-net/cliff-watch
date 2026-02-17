use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use merlin::Transcript;
use rand::thread_rng;
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::ristretto::CompressedRistretto;

/// Representa una prueba de rango ZKP que demuestra que un Score de Humanidad
/// está por encima de un umbral sin revelar el score exacto.
#[derive(Debug)]
pub struct HumanityProof {
    pub commitment: CompressedRistretto,
    pub proof: RangeProof,
}

impl HumanityProof {
    /// Genera una prueba de rango para un score dado.
    pub fn generate(score_percent: u64, threshold_percent: u64) -> Result<Self, String> {
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);
        
        if score_percent < threshold_percent {
            return Err("Score below threshold, cannot generate proof".into());
        }
        
        let secret_value = score_percent - threshold_percent;
        let mut rng = thread_rng();
        let mut transcript = Transcript::new(b"cliff-watch-pohw");
        
        let blinding = Scalar::random(&mut rng);
        
        let (proof, commitment) = RangeProof::prove_single(
            &bp_gens,
            &pc_gens,
            &mut transcript,
            secret_value,
            &blinding,
            64,
        ).map_err(|e| format!("ZKP Proof generation failed: {}", e))?;
        
        Ok(Self {
            commitment,
            proof,
        })
    }

    /// Verifica si la prueba es válida.
    pub fn verify(&self) -> Result<(), String> {
        let pc_gens = PedersenGens::default();
        let bp_gens = BulletproofGens::new(64, 1);
        let mut transcript = Transcript::new(b"cliff-watch-pohw");
        
        self.proof.verify_single(
            &bp_gens,
            &pc_gens,
            &mut transcript,
            &self.commitment,
            64,
        ).map_err(|e| format!("ZKP Verification failed: {}", e))?;
        
        Ok(())
    }
}
