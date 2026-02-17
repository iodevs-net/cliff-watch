use std::path::Path;
use std::process;

#[derive(clap::Subcommand, Debug)]
pub enum ConfigAction {
    /// Crea un archivo de configuración por defecto
    Init,
    /// Valida la configuración actual
    Check,
}

/// Handle the config command - Configuration management
pub fn handle_config(action: ConfigAction) {
    match action {
        ConfigAction::Init => {
            let config_content = r#"# Cliff-Watch Configuration

[governance]
# Dificultad de la "Prueba de Sudor" (Easy, Normal, Hardcore)
difficulty = "Normal"
# Entropía mínima para validar un commit (base 2.5)
min_entropy = 2.5

[monitoring]
# Ventana de agrupación de eventos (ms)
debounce_window_ms = 500
# Directorios a ignorar (además de .git)
ignore_top_level_dirs = [".git", "target", "node_modules", "dist", "build"]
# Extensiones de archivo a ignorar
ignore_extensions = ["log", "lock", "tmp", "bak"]
"#;
            let path = Path::new("cliff-watch.toml");
            if path.exists() {
                eprintln!("⚠️ cliff-watch.toml already exists!");
                process::exit(1);
            }
            if let Err(e) = std::fs::write(path, config_content) {
                eprintln!("❌ Failed to create config file: {}", e);
                process::exit(1);
            }
            println!("✅ Created cliff-watch.toml");
        }
        ConfigAction::Check => {
            match cliff_watch_core::config::GovConfig::load() {
                Ok(cfg) => {
                    println!("✅ Configuration valid:");
                    println!("   Difficulty:  {}", cfg.governance.difficulty);
                    println!("   Min Entropy: {}", cfg.governance.min_entropy);
                    println!("   Watch Root:  {}", cfg.monitoring.watch_root);
                }
                Err(e) => {
                    eprintln!("❌ Configuration invalid: {}", e);
                    process::exit(1);
                }
            }
        }
    }
}
