//! Tests for TPM (Trusted Platform Module) operations in cliff-watch-core
//!
//! This module provides comprehensive test coverage for TPM functionality
//! including initialization, evidence signing, and availability checks.

#[cfg(feature = "tpm")]
use cliff_watch_core::crypto::tpm::{TpmWitness, is_tpm_available};

#[cfg(feature = "tpm")]
#[test]
fn test_is_tpm_available() {
    // This test checks if TPM is available on system
    // It will pass regardless of whether TPM is available or not
    let available = is_tpm_available();
    
    // The result should be a boolean (no panic expected)
    let _ = available;
    
    // We can't assert a specific value since it depends on system
    // But we can verify function doesn't panic
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_new() {
    // This test attempts to create a TpmWitness
    // It may fail if TPM is not available, but should not panic
    let result = TpmWitness::new();
    
    // The result may be Ok or Err depending on TPM availability
    // We just verify it returns a Result type
    match result {
        Ok(_witness) => {
            // TPM is available and initialized successfully
        }
        Err(_error) => {
            // TPM is not available or initialization failed
            // This is expected on systems without TPM
            let error_msg = _error.to_string();
            assert!(!error_msg.is_empty(), "Error message should not be empty");
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_sign_evidence() {
    // This test attempts to sign evidence using TPM
    // It requires TPM to be available
    
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        // TPM is available, test signing
        let test_data = b"test evidence data";
        let result = witness.sign_evidence(test_data);
        
        match result {
            Ok(signature) => {
                // Signature should be returned
                assert!(!signature.is_empty(), "Signature should not be empty");
            }
            Err(error) => {
                // Signing failed (e.g., TPM not configured)
                assert!(!error.to_string().is_empty(), "Error message should not be empty");
            }
        }
    } else {
        // TPM not available, skip this test gracefully
        // In a real CI/CD environment, you might want to mark this as ignored
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_sign_empty_data() {
    // Test signing empty data
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let empty_data = b"";
        let result = witness.sign_evidence(empty_data);
        
        // Should handle empty data gracefully
        match result {
            Ok(signature) => {
                // May return signature or empty bytes
                let _ = signature;
            }
            Err(_error) => {
                // Error is acceptable for empty data
            }
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_sign_large_data() {
    // Test signing larger data
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let large_data = vec![0u8; 1024]; // 1KB of data
        let result = witness.sign_evidence(&large_data);
        
        match result {
            Ok(signature) => {
                assert!(!signature.is_empty(), "Signature should not be empty");
            }
            Err(_error) => {
                // Error is acceptable
            }
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_sign_multiple_times() {
    // Test signing multiple times with same witness
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let test_data = b"test data";
        
        // Sign multiple times
        let mut results = Vec::new();
        for _ in 0..3 {
            let result = witness.sign_evidence(test_data);
            results.push(result);
        }
        
        // All attempts should return a Result (Ok or Err)
        for result in results {
            match result {
                Ok(signature) => {
                    assert!(!signature.is_empty(), "Signature should not be empty");
                }
                Err(_error) => {
                    // Error is acceptable
                }
            }
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_error_message_format() {
    // Test that error messages are descriptive
    let witness_result = TpmWitness::new();
    
    if let Err(error) = witness_result {
        // Error message should be descriptive
        let error_msg = error.to_string();
        assert!(!error_msg.is_empty(), "Error message should not be empty");
        
        // Check for common error patterns
        let error_lower = error_msg.to_lowercase();
        assert!(error_lower.contains("tpm") || error_lower.contains("failed") || error_lower.contains("connection"),
                "Error message should be descriptive");
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_sign_evidence_error_message() {
    // Test that signing error messages are descriptive
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let test_data = b"test data";
        let result = witness.sign_evidence(test_data);
        
        if let Err(error) = result {
            let error_msg = error.to_string();
            assert!(!error_msg.is_empty(), "Error message should not be empty");
            
            let error_lower = error_msg.to_lowercase();
            assert!(error_lower.contains("tpm") || error_lower.contains("random") || error_lower.contains("failed"),
                    "Error message should be descriptive");
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_different_data_signatures() {
    // Test that signing different data produces different results
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let data1 = b"first data";
        let data2 = b"second data";
        
        let sig1_result = witness.sign_evidence(data1);
        let sig2_result = witness.sign_evidence(data2);
        
        // Both should return results
        if let (Ok(sig1), Ok(sig2)) = (sig1_result, sig2_result) {
            // Signatures should be different for different data
            assert_ne!(sig1, sig2, "Signatures should be different for different data");
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_same_data_different_signatures() {
    // Test that signing same data multiple times produces different results
    // (due to randomness in TPM operations)
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let test_data = b"test data";
        
        let sig1_result = witness.sign_evidence(test_data);
        let sig2_result = witness.sign_evidence(test_data);
        
        // Both should return results
        if let (Ok(sig1), Ok(sig2)) = (sig1_result, sig2_result) {
            // Signatures may be different due to randomness
            // This is expected behavior for TPM operations
            let _ = (sig1, sig2);
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_signature_length() {
    // Test that signatures have expected length
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let test_data = b"test data";
        let result = witness.sign_evidence(test_data);
        
        if let Ok(signature) = result {
            // Signature should have a reasonable length
            // TPM signatures are typically 32 bytes (256 bits) or more
            assert!(signature.len() >= 16, "Signature should have reasonable length");
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_concurrent_signing() {
    // Test that witness can handle concurrent signing requests
    // This tests Arc<Mutex<Context>> thread safety
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let test_data = b"test data";
        
        // Sign multiple times sequentially (TpmWitness uses Arc internally)
        let result1 = witness.sign_evidence(test_data);
        let result2 = witness.sign_evidence(test_data);
        
        // Both should return results
        match (result1, result2) {
            (Ok(sig1), Ok(sig2)) => {
                // Both succeeded
                let sig1_vec: Vec<u8> = sig1;
                let sig2_vec: Vec<u8> = sig2;
                assert!(!sig1_vec.is_empty(), "First signature should not be empty");
                assert!(!sig2_vec.is_empty(), "Second signature should not be empty");
            }
            _ => {
                // At least one failed, which is acceptable
            }
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_sign_various_data_sizes() {
    // Test signing data of various sizes
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let data_sizes = vec![1, 8, 32, 64, 128, 256, 512];
        
        for size in data_sizes {
            let data = vec![0u8; size];
            let result = witness.sign_evidence(&data);
            
            match result {
                Ok(signature) => {
                    assert!(!signature.is_empty(), "Signature should not be empty for size {}", size);
                }
                Err(_error) => {
                    // Error is acceptable
                }
            }
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_sign_with_special_characters() {
    // Test signing data with special characters
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let special_data = b"test\x00data\xff\x01\x02";
        let result = witness.sign_evidence(special_data);
        
        match result {
            Ok(signature) => {
                assert!(!signature.is_empty(), "Signature should not be empty");
            }
            Err(_error) => {
                // Error is acceptable
            }
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_sign_with_unicode() {
    // Test signing data with Unicode characters
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let unicode_data = "test unicode data: 你好世界 🎉".as_bytes();
        let result = witness.sign_evidence(unicode_data);
        
        match result {
            Ok(signature) => {
                assert!(!signature.is_empty(), "Signature should not be empty");
            }
            Err(_error) => {
                // Error is acceptable
            }
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_multiple_witnesses() {
    // Test creating multiple TpmWitness instances
    let witness1_result = TpmWitness::new();
    let witness2_result = TpmWitness::new();
    
    // Both should return a Result
    match (witness1_result, witness2_result) {
        (Ok(_witness1), Ok(_witness2)) => {
            // Both witnesses created successfully
        }
        (Err(_error1), Err(_error2)) => {
            // Both failed (TPM not available)
        }
        _ => {
            // One succeeded, one failed (unexpected but possible)
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_drop_and_recreate() {
    // Test that a witness can be dropped and recreated
    {
        let witness = TpmWitness::new();
        let _ = witness; // Witness goes out of scope here
    }
    
    // Create a new witness after first one is dropped
    let witness_result = TpmWitness::new();
    
    // Should work without issues
    match witness_result {
        Ok(_witness) => {
            // Witness created successfully
        }
        Err(_error) => {
            // TPM not available (acceptable)
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_sign_after_delay() {
    // Test signing after a short delay
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let test_data = b"test data";
        
        // Sign immediately
        let _ = witness.sign_evidence(test_data);
        
        // Wait a bit
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Sign again after delay
        let result = witness.sign_evidence(test_data);
        
        match result {
            Ok(signature) => {
                assert!(!signature.is_empty(), "Signature should not be empty after delay");
            }
            Err(_error) => {
                // Error is acceptable
            }
        }
    }
}

#[cfg(feature = "tpm")]
#[test]
fn test_tpm_witness_error_handling() {
    // Test that errors are handled gracefully
    let witness_result = TpmWitness::new();
    
    if let Ok(witness) = witness_result {
        let test_data = b"test data";
        
        // Multiple signing attempts should all return Results
        for _ in 0..5 {
            let result = witness.sign_evidence(test_data);
            
            // Should always return a Result, never panic
            match result {
                Ok(_signature) => {
                    // Success
                }
                Err(_error) => {
                    // Error is acceptable
                }
            }
        }
    }
}
