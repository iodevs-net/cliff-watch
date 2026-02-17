use cliff_watch_core::{git::{open_repository, get_trusted_keys}, crypto::{verify_signature, VerifyingKey}};
use std::path::Path;
use std::process;

#[derive(serde::Serialize)]
struct VerificationReport {
    status: String,
    commit: String,
    signer: Option<String>,
    score: Option<f64>,
    reason: Option<String>,
}

/// Handle the verify command - Verify commit signatures
pub fn handle_verify(commit: String, format: String) {
    let repo = match open_repository(Path::new(".")) {
        Ok(repo) => repo,
        Err(e) => {
            let report = VerificationReport {
                status: "error".to_string(),
                commit: commit.clone(),
                signer: None,
                score: None,
                reason: Some(format!("Error opening repository: {}", e)),
            };
            if format == "json" {
                println!("{}", serde_json::to_string(&report)
                    .expect("Failed to serialize verification report to JSON"));
            } else {
                eprintln!("❌ Error opening repository: {}", e);
            }
            process::exit(1);
        }
    };

    let commit_obj = match repo.revparse_single(&commit) {
        Ok(obj) => obj.peel_to_commit()
            .expect("Failed to peel object to commit"),
        Err(e) => {
            let report = VerificationReport {
                status: "error".to_string(),
                commit: commit.clone(),
                signer: None,
                score: None,
                reason: Some(format!("Commit not found: {}", e)),
            };
            if format == "json" {
                println!("{}", serde_json::to_string(&report)
                    .expect("Failed to serialize verification report to JSON"));
            } else {
                eprintln!("❌ Commit not found: {}", e);
            }
            process::exit(1);
        }
    };

    let message = commit_obj.message().unwrap_or("");
    let trusted_keys = get_trusted_keys(&repo).unwrap_or_default();

    // Buscar trailer de cliff-watch
    let mut found = false;
    for line in message.lines() {
        if line.starts_with("cliff-watch-score:") {
            let score_part = line.replace("cliff-watch-score:", "").trim().to_string();
            // El formato esperado es "score=0.85:sig=<hex>"
            let parts: Vec<&str> = score_part.split(":sig=").collect();
            if parts.len() == 2 {
                let score_str = parts[0].replace("score=", "");
                let sig_hex = parts[1];
                let sig_bytes = hex::decode(sig_hex).unwrap_or_default();

                let mut verified = false;
                let mut signer_alias = "Unknown";

                for (alias, key_hex) in &trusted_keys {
                    let key_bytes = hex::decode(key_hex).unwrap_or_default();
                    if let Ok(verifying_key) = VerifyingKey::from_bytes(&key_bytes.try_into().unwrap_or([0;32])) {
                        if verify_signature(&verifying_key, parts[0].as_bytes(), &sig_bytes).unwrap_or(false) {
                            verified = true;
                            signer_alias = alias;
                            break;
                        }
                    }
                }

                if verified {
                    let score_val = score_str.parse::<f64>().ok();
                    if format == "json" {
                        let report = VerificationReport {
                            status: "verified".to_string(),
                            commit: commit.clone(),
                            signer: Some(signer_alias.to_string()),
                            score: score_val,
                            reason: None,
                        };
                        println!("{}", serde_json::to_string(&report)
                            .expect("Failed to serialize verification report to JSON"));
                    } else {
                        println!("✅ Commit VERIFICADO Criptográficamente");
                        println!("   Firmante: {}", signer_alias);
                        println!("   Score:    {}", score_str);
                    }
                    found = true;
                }
            }
        }
    }

    if !found {
        if format == "json" {
             let report = VerificationReport {
                status: "failed".to_string(),
                commit: commit.clone(),
                signer: None,
                score: None,
                reason: Some("no_valid_signature".to_string()),
            };
            println!("{}", serde_json::to_string(&report)
                .expect("Failed to serialize verification report to JSON"));
        } else {
            eprintln!("❌ FALLO DE VERIFICACIÓN: No se encontró firma válida de Cliff-Watch.");
        }
        process::exit(1);
    }
}
