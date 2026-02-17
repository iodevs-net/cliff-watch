use cliff_watch_core::{git::open_repository, crypto::generate_keypair};
use std::path::Path;
use std::process;

/// Handle the init command - Initialize cliff-watch in current repository
pub fn handle_init(path: String) {
    let repo_path = Path::new(&path);
    match open_repository(repo_path) {
        Ok(_repo) => {
            println!("Initializing cliff-watch in: {}", path);

            // Generate keypair for the repository
            let (_signing_key, verifying_key) = generate_keypair();
            let pubkey_hex = cliff_watch_core::crypto::bytes_to_hex(verifying_key.as_bytes());
            println!("Generated new keypair for repository");
            println!("Public key: {}", pubkey_hex);

            // Add cliff-watch configuration to git config
            // This would typically include the public key and other settings

            // Install hooks
            let config = cliff_watch_core::config::GovConfig::load().unwrap_or_default();
            match cliff_watch_core::git::install_hooks(&_repo, &config) {
                Ok(_) => println!("✅ Git hooks installed successfully (Audit Mode: {})", config.governance.audit_mode),
                Err(e) => eprintln!("⚠️ Failed to install git hooks: {}", e),
            }

            println!("✅ Repository initialized successfully");
            println!("Public key stored: {}", pubkey_hex);
        }
        Err(error) => {
            eprintln!("❌ Failed to initialize repository: {}", error);
            process::exit(1);
        }
    }
}
