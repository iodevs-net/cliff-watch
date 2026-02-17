use clap::{Parser, Subcommand};
use cliff_watch_core::{sentinel_self_check, git::open_repository};
use std::process::{self, Command, Stdio};
use std::path::Path;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use console::{style, Emoji};
use std::time::Duration;

// Import command modules
mod commands;
use commands::{handle_config, handle_daemon, handle_off, handle_on, handle_init, handle_metrics, handle_report, handle_verify};

#[derive(Parser, Debug)]
#[command(
    name = "cliff-watch",
    about = "Decentralized Code Governance (DCG) tool implementing Proof of Human Work (PoHW)",
    version,
    author
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static SPARKLES: Emoji<'_, '_> = Emoji("✨  ", "");
static GEAR: Emoji<'_, '_> = Emoji("⚙️  ", "");
static PACKAGE: Emoji<'_, '_> = Emoji("📦  ", "");
static SUCCESS: Emoji<'_, '_> = Emoji("✅  ", "");

#[derive(Subcommand, Debug)]
enum Commands {
    /// System integrity check
    SystemCheck {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Initialize cliff-watch in current repository
    Init {
        /// Repository path (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    /// Desactiva cliff-watch en el repositorio actual (elimina hooks)
    Disable {
        /// Repository path (default: current directory)
        #[arg(short, long, default_value = ".")]
        path: String,
    },
    /// Start the cliff-watch daemon
    Daemon {
        /// Configuration file path
        #[arg(short, long, default_value = "config.toml")]
        config: String,
        /// Run in background
        #[arg(short, long)]
        daemon: bool,
    },
    /// Check daemon status
    Status,
    /// Activa el centinela (inicia el daemon en background)
    On,
    /// Desactiva el centinela (detiene el daemon)
    Off,
    /// View real-time kinematic metrics
    Metrics {
        /// Output only the human score (short format)
        #[arg(short, long)]
        short: bool,
    },
    /// Verificación commit-por-commit (para CI/CD o auditorías)
    Verify {
        /// Hash o referencia del commit
        #[arg(default_value = "HEAD")]
        commit: String,
        /// Formato de salida (json, text)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Registra una clave pública para verificación en este repositorio
    RegisterKey {
        /// Clave pública en formato hexadecimal
        #[arg(short, long)]
        key: String,
        /// Alias para la clave (ej: nombre del dev)
        #[arg(short, long)]
        alias: String,
    },
    /// Verificación termodinámica del trabajo (para hooks)
    VerifyWork,
    /// Genera la evidencia técnica (trailers) para el commit actual
    Inspect {
        /// Ruta al archivo de mensaje de commit (pasado por Git)
        #[arg()]
        message_file: Option<String>,
    },
    /// Gestión de configuración
    Config {
        #[command(subcommand)]
        action: commands::config::ConfigAction,
    },
    /// Genera reporte de auditoría
    Report {
        /// Número de commits a analizar
        #[arg(short, long, default_value_t = 100)]
        limit: usize,
        /// Formato de salida (text, json, md)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Configura el entorno de desarrollo de forma automática y premium
    Setup {
        /// Evita la interacción con el usuario
        #[arg(short, long)]
        yes: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Setup { yes } => {
            run_setup(yes).await;
        }
        Commands::Report { limit, format } => {
            handle_report(limit, format);
        }
        Commands::SystemCheck { verbose } => {
            match sentinel_self_check() {
                Ok(report) => {
                    if verbose {
                        println!("System Check Report:\n{}", report);
                    } else {
                        println!("✅ System integrity check passed");
                    }
                    process::exit(0);
                }
                Err(error) => {
                    eprintln!("❌ System integrity check failed: {}", error);
                    process::exit(1);
                }
            }
        }
        Commands::Init { path } => {
            handle_init(path);
        }
        Commands::Disable { path } => {
            let repo_path = Path::new(&path);
            match open_repository(repo_path) {
                Ok(repo) => {
                    match cliff_watch_core::git::remove_hooks(&repo) {
                        Ok(_) => println!("✅ Cliff-Watch disabled: Hooks removed successfully"),
                        Err(e) => eprintln!("❌ Failed to remove hooks: {}", e),
                    }
                }
                Err(e) => {
                    eprintln!("❌ Failed to open repository: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Daemon { config, daemon } => {
            handle_daemon(config, daemon);
        }
        Commands::Status => {
            commands::handle_status().await;
        }
        Commands::On => {
            handle_on();
        }
        Commands::Off => {
            handle_off();
        }
        Commands::Metrics { short } => {
            handle_metrics(short).await;
        }
        Commands::RegisterKey { key, alias } => {
            let repo = match open_repository(Path::new(".")) {
                Ok(repo) => repo,
                Err(e) => {
                    eprintln!("❌ Error opening repository: {}", e);
                    process::exit(1);
                }
            };

            match cliff_watch_core::git::register_public_key(&repo, &key, &alias) {
                Ok(_) => println!("✅ Key registered successfully for alias: {}", alias),
                Err(e) => {
                    eprintln!("❌ Failed to register key: {}", e);
                    process::exit(1);
                }
            }
        }
        Commands::Verify { commit, format } => {
            handle_verify(commit, format);
        }
        Commands::VerifyWork => {
            use cliff_watch_core::git::get_staged_diff;
            use cliff_watch_core::complexity::estimate_entropic_cost;

            let repo = match open_repository(Path::new(".")) {
                Ok(repo) => repo,
                Err(e) => {
                    eprintln!("❌ Error opening repository: {}", e);
                    process::exit(1);
                }
            };

            let diff = match get_staged_diff(&repo) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("❌ Error getting staged diff: {}", e);
                    process::exit(1);
                }
            };

            if diff.is_empty() {
                process::exit(0);
            }

            let cost = estimate_entropic_cost(&diff, None);

            match query_daemon(cliff_watch_core::protocol::Request::GetTicket { cost }).await {
                Ok(cliff_watch_core::protocol::Response::Ticket { success, message, signature }) => {
                    if success {
                        println!("✅ Thermodynamic check passed: {}", message);

                        // Guardar el ticket firmado para el hook prepare-commit-msg
                        if let Some(sig_bytes) = signature {
                            let sig_hex = hex::encode(sig_bytes);
                            let ticket_data = format!("score={:.2}:sig={}", cost, sig_hex);

                            let gov_dir = repo.path().join("cliff-watch");
                            if !gov_dir.exists() {
                                let _ = std::fs::create_dir_all(&gov_dir);
                            }
                            let ticket_file = gov_dir.join("latest_ticket");
                            if let Err(e) = std::fs::write(ticket_file, ticket_data) {
                                eprintln!("⚠️ Error saving ticket: {}", e);
                            }
                        }

                        // v2.0: Obtener datos del Witness para certificación de foco
                        match query_daemon(cliff_watch_core::protocol::Request::GetWitness { reset: true }).await {
                            Ok(cliff_watch_core::protocol::Response::Witness { data }) => {
                                let witness_file = repo.path().join("cliff-watch").join("latest_witness");
                                if let Err(e) = std::fs::write(witness_file, data) {
                                    eprintln!("⚠️ Error saving witness data: {}", e);
                                } else {
                                    println!("✅ Focus witness data recorded (v2.0)");
                                }
                            }
                            _ => eprintln!("⚠️ Could not retrieve focus witness data"),
                        }

                        process::exit(0);
                    } else {
                        eprintln!("❌ {}", message);
                        process::exit(1);
                    }
                }
                Ok(cliff_watch_core::protocol::Response::Error(e)) => {
                    eprintln!("❌ Daemon error: {}", e);
                    process::exit(1);
                }
                Err(e) => {
                    eprintln!("❌ Daemon communication error: {}", e);
                    process::exit(1);
                }
                _ => {
                    eprintln!("❌ Unexpected response from daemon");
                    process::exit(1);
                }
            }
        }
        Commands::Inspect { message_file } => {
            let repo = match open_repository(Path::new(".")) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("❌ Error opening repository: {}", e);
                    process::exit(1);
                }
            };

            // 1. Obtener la evidencia del Witness (reseteando el acumulador)
            match query_daemon(cliff_watch_core::protocol::Request::GetWitness { reset: true }).await {
                Ok(cliff_watch_core::protocol::Response::Witness { data }) => {
                    let gov_dir = repo.path().join("cliff-watch");
                    if !gov_dir.exists() {
                        let _ = std::fs::create_dir_all(&gov_dir);
                    }

                    let witness_file = gov_dir.join("latest_witness");
                    if let Err(e) = std::fs::write(&witness_file, &data) {
                        eprintln!("⚠️ Error saving witness data: {}", e);
                    } else {
                        println!("✅ Evidence generated: Cliff-Watch-Witness v2.0");
                    }

                    // 2. Si se proporciona un archivo de mensaje (Git hook manual), inyectamos ahora
                    if let Some(msg_file_path) = message_file {
                        let msg_path = Path::new(&msg_file_path);
                        if msg_path.exists() {
                            let _ = Command::new("git")
                                .args(["interpret-trailers", "--in-place", "--trailer", &format!("Cliff-Watch-Witness: {}", data), &msg_file_path])
                                .status();
                        }
                    }
                }
                _ => {
                    eprintln!("❌ Could not retrieve witness evidence from daemon.");
                    process::exit(1);
                }
            }
        }
        Commands::Config { action } => {
            handle_config(action);
        }
    }
}

async fn query_daemon(request: cliff_watch_core::protocol::Request) -> anyhow::Result<cliff_watch_core::protocol::Response> {
    use tokio::net::UnixStream;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let socket_path = "/tmp/cliff-watch.sock";
    let mut stream = UnixStream::connect(socket_path).await?;

    let request_json = serde_json::to_vec(&request)?;
    stream.write_all(&request_json).await?;

    let mut buffer = vec![0; 1024];
    let n = stream.read(&mut buffer).await?;

    let response: cliff_watch_core::protocol::Response = serde_json::from_slice(&buffer[..n])?;
    Ok(response)
}

async fn run_setup(no_confirm: bool) {
    println!("\n{} {} {}", style("===").blue(), style("Orquestador Soberano cliff-watch v2.1").bold(), style("===").blue());
    println!("{}\n", style("Preparando tu PC para la verdadera Gobernanza de Código...").italic());

    if !no_confirm {
        println!("Este proceso instalará dependencias de sistema y herramientas globales.");
        println!("Se requiere acceso de administrador (sudo) para algunos pasos.");
    }

    let m = MultiProgress::new();
    let sty = ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
        .expect("Failed to create progress style template")
        .progress_chars("##-");

    // Paso 1: Dependencias de Sistema (APT/Pacman)
    let pb1 = m.add(ProgressBar::new(1));
    pb1.set_style(sty.clone());
    pb1.set_message("Escrutando dependencias del sistema operativo...");

    tokio::time::sleep(Duration::from_millis(500)).await;
    pb1.set_message(format!("{} Instalando dependencias base (cmake, ssl, zstd)...", PACKAGE));

    let mut cmd = if Path::new("/etc/debian_version").exists() {
        let mut c = Command::new("sudo");
        c.args(&["apt", "install", "-y", "build-essential", "pkg-config", "libssl-dev", "libzstd-dev", "libtss2-dev", "cmake", "curl", "git"]);
        c
    } else {
        let mut c = Command::new("sudo");
        c.args(&["pacman", "-S", "--needed", "--noconfirm", "base-devel", "pkgconf", "openssl", "zstd", "tpm2-tss", "cmake", "curl", "git"]);
        c
    };

    let status = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status();
    if status.is_err() || !status.as_ref().map(|s| s.success()).unwrap_or(false) {
        pb1.abandon_with_message("❌ Error instalando dependencias de sistema.");
    } else {
        pb1.finish_with_message(format!("{} Dependencias de sistema listas.", SUCCESS));
    }

    // Paso 2: Herramientas Globales de Cargo
    let tools = vec!["cargo-audit", "cargo-expand", "cargo-nextest"];
    let pb2 = m.add(ProgressBar::new(tools.len() as u64));
    pb2.set_style(sty.clone());

    for tool in tools {
        pb2.set_message(format!("{} Forjando herramienta: {}...", GEAR, tool));
        let _ = Command::new("cargo")
            .args(&["install", tool, "--quiet"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        pb2.inc(1);
    }
    pb2.finish_with_message(format!("{} Armería de Cargo completa.", SUCCESS));

    // Paso 3: Extensiones de VSCode (Modo Soberano)
    let pb3 = m.add(ProgressBar::new(1));
    pb3.set_style(sty);
    pb3.set_message(format!("{} Forjando el Testigo (VSIX local)...", LOOKING_GLASS));

    // Intentar instalar VSCode extension localmente
    let vsix_built = Command::new("sh")
        .arg("-c")
        .arg("cd clients/cliff-watch-witness && npm install && npx vsce package --out ../../cliff-watch-witness.vsix --no-interaction")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if vsix_built.as_ref().map(|s| s.success()).unwrap_or(false) {
        pb3.set_message(format!("{} Instalando Testigo Soberano...", SUCCESS));
        let _ = Command::new("code")
            .args(&["--install-extension", "cliff-watch-witness.vsix", "--force"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        pb3.finish_with_message(format!("{} Cliff-Watch Witness activado (Local).", SUCCESS));
    } else {
        pb3.abandon_with_message("⚠️ Error al forjar el Testigo local. Intenta manualmente con 'cliff-watch setup'.");
    }

    println!("\n{} {}", SPARKLES, style("¡Misión cumplida! Tu PC ahora es Soberano.").green().bold());
    println!("Ejecuta {} para empezar a validar tu entropía.", style("cliff-watch on").cyan());
}
