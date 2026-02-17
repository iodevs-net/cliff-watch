#[cfg(feature = "zkp")]
pub mod zkp;
#[cfg(feature = "tpm")]
pub mod tpm;
pub use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::fs;
use std::io::{Read, Write};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Errors that can occur during cryptographic operations
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    /// Error signing data
    #[error("Failed to sign data")]
    SigningError,

    /// Error verifying signature
    #[error("Failed to verify signature")]
    VerificationError,

    /// Invalid signature length (expected 64 bytes)
    #[error("Invalid signature length: expected 64 bytes, got {0}")]
    InvalidSignatureLength(usize),

    /// Invalid signature format
    #[error("Invalid signature format")]
    InvalidSignatureFormat,

    /// Error loading identity key
    #[error("Failed to load identity: {0}")]
    IdentityLoadError(String),

    /// Error generating keypair
    #[error("Failed to generate keypair")]
    KeypairGenerationError,

    /// Error reading key file
    #[error("Failed to read key file: {0}")]
    KeyFileReadError(String),

    /// Error writing key file
    #[error("Failed to write key file: {0}")]
    KeyFileWriteError(String),

    /// Error creating config directory
    #[error("Failed to create config directory: {0}")]
    ConfigDirError(String),

    /// Invalid key file size
    #[error("Invalid key file size: expected 32 bytes, got {0}")]
    InvalidKeySize(usize),

    /// Error setting file permissions
    #[error("Failed to set file permissions: {0}")]
    PermissionsError(String),

    /// Environment variable not found
    #[error("Environment variable not found: {0}")]
    EnvVarNotFound(String),
}

/// Genera un par de claves Ed25519
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Firma datos usando una clave privada Ed25519
pub fn sign_data(signing_key: &SigningKey, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let signature = signing_key.sign(data);
    Ok(signature.to_bytes().to_vec())
}

/// Verifica una firma usando una clave pública Ed25519
pub fn verify_signature(verifying_key: &VerifyingKey, data: &[u8], signature: &[u8]) -> Result<bool, CryptoError> {
    if signature.len() != 64 {
        return Err(CryptoError::InvalidSignatureLength(signature.len()));
    }
    let signature_bytes: [u8; 64] = signature.try_into()
        .map_err(|_| CryptoError::InvalidSignatureFormat)?;
    let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes);
    Ok(verifying_key.verify(data, &signature).is_ok())
}

/// Calcula el hash SHA256 de los datos
pub fn calculate_sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Convierte un slice de bytes a su representación hexadecimal
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut acc, &byte| {
        acc.push_str(&format!("{:02x}", byte));
        acc
    })
}

/// Carga la identidad del daemon desde ~/.config/cliff-watch/daemon.key o crea una nueva
pub fn load_or_create_identity() -> Result<SigningKey, CryptoError> {
    let home = std::env::var("HOME")
        .map_err(|_| CryptoError::EnvVarNotFound("HOME".to_string()))?;
    let config_dir = PathBuf::from(home).join(".config").join("cliff-watch");
    let key_path = config_dir.join("daemon.key");

    if !config_dir.exists() {
        fs::create_dir_all(&config_dir)
            .map_err(|e| CryptoError::ConfigDirError(e.to_string()))?;
    }

    if key_path.exists() {
        let mut file = fs::File::open(&key_path)
            .map_err(|e| CryptoError::KeyFileReadError(e.to_string()))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .map_err(|e| CryptoError::KeyFileReadError(e.to_string()))?;
        
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeySize(bytes.len()));
        }

        let key_bytes: [u8; 32] = bytes.try_into()
            .map_err(|_| CryptoError::InvalidSignatureFormat)?;
        Ok(SigningKey::from_bytes(&key_bytes))
    } else {
        let (signing_key, _) = generate_keypair();
        let bytes = signing_key.to_bytes();
        
        let mut file = fs::File::create(&key_path)
            .map_err(|e| CryptoError::KeyFileWriteError(e.to_string()))?;
        
        #[cfg(unix)]
        {
            let mut perms = file.metadata()
                .map_err(|e| CryptoError::PermissionsError(e.to_string()))?
                .permissions();
            perms.set_mode(0o600);
            file.set_permissions(perms)
                .map_err(|e| CryptoError::PermissionsError(e.to_string()))?;
        }

        file.write_all(&bytes)
            .map_err(|e| CryptoError::KeyFileWriteError(e.to_string()))?;
        Ok(signing_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sign_verify() {
        let (signing_key, verifying_key) = generate_keypair();
        let data = b"test data";
        let signature = sign_data(&signing_key, data)
            .expect("Failed to sign test data");
        let is_valid = verify_signature(&verifying_key, data, &signature)
            .expect("Failed to verify signature");
        assert!(is_valid, "Signature should be valid");
    }
    
    #[test]
    fn test_sha256() {
        let data = b"test";
        let hash = calculate_sha256(data);
        assert_eq!(hash.len(), 32, "SHA256 hash should be 32 bytes");
    }
}