//! Tests for Zero-Knowledge Proof (ZKP) verification in cliff-watch-core
//!
//! This module provides comprehensive test coverage for ZKP functionality
//! including proof generation, verification, and edge cases.

#[cfg(feature = "zkp")]
use cliff_watch_core::crypto::zkp::HumanityProof;

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_generate_valid() {
    let score_percent = 85;
    let threshold_percent = 50;
    
    let result = HumanityProof::generate(score_percent, threshold_percent);
    assert!(result.is_ok(), "Should generate proof when score >= threshold");
    
    let proof = result.unwrap();
    // Verify the proof is valid
    let verify_result = proof.verify();
    assert!(verify_result.is_ok(), "Generated proof should be valid");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_generate_at_threshold() {
    let score_percent = 50;
    let threshold_percent = 50;
    
    let result = HumanityProof::generate(score_percent, threshold_percent);
    assert!(result.is_ok(), "Should generate proof when score == threshold");
    
    let proof = result.unwrap();
    let verify_result = proof.verify();
    assert!(verify_result.is_ok(), "Proof at threshold should be valid");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_generate_below_threshold() {
    let score_percent = 30;
    let threshold_percent = 50;
    
    let result = HumanityProof::generate(score_percent, threshold_percent);
    assert!(result.is_err(), "Should fail to generate proof when score < threshold");
    
    let error = result.unwrap_err();
    assert!(error.contains("Score below threshold"), "Error message should indicate score below threshold");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_verify_success() {
    let score_percent = 75;
    let threshold_percent = 50;
    
    let proof = HumanityProof::generate(score_percent, threshold_percent)
        .expect("Failed to generate proof");
    
    let verify_result = proof.verify();
    assert!(verify_result.is_ok(), "Proof verification should succeed");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_verify_high_score() {
    let score_percent = 95;
    let threshold_percent = 60;
    
    let proof = HumanityProof::generate(score_percent, threshold_percent)
        .expect("Failed to generate proof");
    
    let verify_result = proof.verify();
    assert!(verify_result.is_ok(), "High score proof should verify successfully");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_verify_low_valid_score() {
    let score_percent = 51;
    let threshold_percent = 50;
    
    let proof = HumanityProof::generate(score_percent, threshold_percent)
        .expect("Failed to generate proof");
    
    let verify_result = proof.verify();
    assert!(verify_result.is_ok(), "Low valid score proof should verify successfully");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_commitment_exists() {
    let score_percent = 80;
    let threshold_percent = 50;
    
    let proof = HumanityProof::generate(score_percent, threshold_percent)
        .expect("Failed to generate proof");
    
    // Check that commitment is not default (zero) value
    let commitment_bytes = proof.commitment.as_bytes();
    let is_zero = commitment_bytes.iter().all(|&b| b == 0);
    assert!(!is_zero, "Commitment should not be all zeros");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_multiple_proofs() {
    let threshold_percent = 50;
    
    // Generate multiple proofs with different scores
    let proofs: Vec<_> = vec![55, 60, 70, 80, 90]
        .iter()
        .map(|&score| HumanityProof::generate(score, threshold_percent))
        .collect();
    
    // All proofs should be generated successfully
    for result in proofs {
        assert!(result.is_ok(), "Should generate proof for each score");
    }
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_different_thresholds() {
    let score_percent = 75;
    
    // Generate proofs with different thresholds
    let thresholds = vec![50, 60, 70];
    
    for threshold in thresholds {
        if score_percent >= threshold {
            let result = HumanityProof::generate(score_percent, threshold);
            assert!(result.is_ok(), "Should generate proof when score >= threshold");
            
            let proof = result.unwrap();
            let verify_result = proof.verify();
            assert!(verify_result.is_ok(), "Proof should verify successfully");
        }
    }
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_edge_case_zero_threshold() {
    let score_percent = 10;
    let threshold_percent = 0;
    
    let result = HumanityProof::generate(score_percent, threshold_percent);
    assert!(result.is_ok(), "Should generate proof when threshold is 0");
    
    let proof = result.unwrap();
    let verify_result = proof.verify();
    assert!(verify_result.is_ok(), "Proof with zero threshold should verify");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_edge_case_max_score() {
    let score_percent = 100;
    let threshold_percent = 50;
    
    let result = HumanityProof::generate(score_percent, threshold_percent);
    assert!(result.is_ok(), "Should generate proof for max score");
    
    let proof = result.unwrap();
    let verify_result = proof.verify();
    assert!(verify_result.is_ok(), "Max score proof should verify");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_generate_verify_cycle() {
    let score_percent = 85;
    let threshold_percent = 50;
    
    // Generate proof
    let proof = HumanityProof::generate(score_percent, threshold_percent)
        .expect("Failed to generate proof");
    
    // Verify proof
    let verify_result = proof.verify();
    assert!(verify_result.is_ok(), "Proof should verify successfully");
    
    // Verify multiple times (idempotency)
    for _ in 0..5 {
        let verify_result = proof.verify();
        assert!(verify_result.is_ok(), "Proof should verify multiple times");
    }
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_deterministic_verification() {
    let score_percent = 75;
    let threshold_percent = 50;
    
    let proof = HumanityProof::generate(score_percent, threshold_percent)
        .expect("Failed to generate proof");
    
    // Verify multiple times and ensure consistent results
    let mut results = Vec::new();
    for _ in 0..10 {
        let verify_result = proof.verify();
        results.push(verify_result.is_ok());
    }
    
    assert!(results.iter().all(|&r| r), "All verification attempts should succeed");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_error_message() {
    let score_percent = 30;
    let threshold_percent = 50;
    
    let result = HumanityProof::generate(score_percent, threshold_percent);
    assert!(result.is_err());
    
    let error = result.unwrap_err();
    assert!(!error.is_empty(), "Error message should not be empty");
    assert!(error.contains("below threshold") || error.contains("cannot generate"), 
            "Error message should be descriptive");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_verify_error_message() {
    // Create a proof with a valid score
    let proof = HumanityProof::generate(80, 50)
        .expect("Failed to generate proof");
    
    // The verification should succeed, but if we had an invalid proof,
    // error message should be descriptive
    let verify_result = proof.verify();
    if verify_result.is_err() {
        let error = verify_result.unwrap_err();
        assert!(!error.is_empty(), "Error message should not be empty");
        assert!(error.contains("Verification failed") || error.contains("ZKP"), 
                "Error message should be descriptive");
    }
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_structure() {
    let score_percent = 80;
    let threshold_percent = 50;
    
    let proof = HumanityProof::generate(score_percent, threshold_percent)
        .expect("Failed to generate proof");
    
    // Check that proof has expected structure
    let commitment_bytes = proof.commitment.as_bytes();
    assert_eq!(commitment_bytes.len(), 32, "Commitment should be 32 bytes (Ristretto compressed)");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_score_above_threshold_variations() {
    let threshold_percent = 50;
    
    // Test various scores above threshold
    let scores_above = vec![51, 55, 60, 65, 70, 75, 80, 85, 90, 95, 99, 100];
    
    for score in scores_above {
        let result = HumanityProof::generate(score, threshold_percent);
        assert!(result.is_ok(), "Should generate proof for score {}", score);
        
        let proof = result.unwrap();
        let verify_result = proof.verify();
        assert!(verify_result.is_ok(), "Proof for score {} should verify", score);
    }
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_score_below_threshold_variations() {
    let threshold_percent = 50;
    
    // Test various scores below threshold
    let scores_below = vec![0, 1, 10, 20, 30, 40, 49];
    
    for score in scores_below {
        let result = HumanityProof::generate(score, threshold_percent);
        assert!(result.is_err(), "Should fail to generate proof for score {}", score);
    }
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_threshold_variations() {
    let score_percent = 75;
    
    // Test various thresholds
    let thresholds = vec![0, 10, 25, 50, 60, 70, 75];
    
    for threshold in thresholds {
        if score_percent >= threshold {
            let result = HumanityProof::generate(score_percent, threshold);
            assert!(result.is_ok(), "Should generate proof for threshold {}", threshold);
            
            let proof = result.unwrap();
            let verify_result = proof.verify();
            assert!(verify_result.is_ok(), "Proof for threshold {} should verify", threshold);
        } else {
            let result = HumanityProof::generate(score_percent, threshold);
            assert!(result.is_err(), "Should fail for threshold {} above score", threshold);
        }
    }
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_commitment_uniqueness() {
    let score_percent = 80;
    let threshold_percent = 50;
    
    // Generate two proofs with same parameters
    let proof1 = HumanityProof::generate(score_percent, threshold_percent)
        .expect("Failed to generate first proof");
    let proof2 = HumanityProof::generate(score_percent, threshold_percent)
        .expect("Failed to generate second proof");
    
    // Commitments should be different due to random blinding
    assert_ne!(proof1.commitment.as_bytes(), proof2.commitment.as_bytes(), 
               "Commitments should be different due to random blinding");
    
    // Both should verify successfully
    assert!(proof1.verify().is_ok(), "First proof should verify");
    assert!(proof2.verify().is_ok(), "Second proof should verify");
}

#[cfg(feature = "zkp")]
#[test]
fn test_humanity_proof_commitment_bytes() {
    let score_percent = 80;
    let threshold_percent = 50;
    
    let proof = HumanityProof::generate(score_percent, threshold_percent)
        .expect("Failed to generate proof");
    
    // Verify commitment bytes are accessible
    let commitment_bytes = proof.commitment.as_bytes();
    assert_eq!(commitment_bytes.len(), 32, "Commitment should be 32 bytes");
}
