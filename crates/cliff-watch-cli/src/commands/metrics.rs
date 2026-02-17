use console::style;
use std::process;

/// Handle the metrics command - View real-time kinematic metrics
pub async fn handle_metrics(short: bool) {
    match query_daemon(cliff_watch_core::protocol::Request::GetMetrics).await {
        Ok(cliff_watch_core::protocol::Response::Metrics {
            ldlj: _, entropy: _, throughput: _, human_score, coupling, battery_level,
            focus_time_mins, edit_bursts, is_focused, zkp_proof, score_history
        }) => {
            if short {
                println!("{:.4}", human_score);
            } else {
                // Generate Sparkline
                let bars = " ▂▃▄▅▆▇█";
                let history_viz: String = score_history.iter().map(|&v| {
                    let idx = ((v.max(0.0).min(1.0) * 8.0) as usize).min(7);
                    let bar = bars.chars().nth(idx).unwrap_or(' ');
                    // Colorize: <0.4 Red, <0.7 Yellow, >=0.7 Green
                    if v < 0.4 {
                        format!("{}", style(bar).red())
                    } else if v < 0.7 {
                        format!("{}", style(bar).yellow())
                    } else {
                        format!("{}", style(bar).green())
                    }
                }).collect();

                println!("GovMonitor - Estado Termodinámico v2.1:");
                println!("  🔋 Energía (Kinética+Foco): {:.1}%", battery_level);
                println!("  🧠 Acoplamiento Cognitivo:  {:.1}%", coupling * 100.0);
                println!("  --------------------------------");
                println!("  ⏱️  Tiempo de Foco:         {:.2} min", focus_time_mins);
                println!("  ✍️  Ráfagas de Edición:     {}", edit_bursts);
                println!("  👁️  Sensor IDE:             {}", if is_focused { "✅ ACTIVO" } else { "💤 INACTIVO" });
                println!("  --------------------------------");
                println!("  📊 Timeline de Humanidad (3 min):");
                println!("     [{}]", history_viz);
                println!("  --------------------------------");
                println!("  🛡️  Human Probability:     {:.2}%", human_score * 100.0);
                println!("  🔐  ZKP Proof:             {}", if zkp_proof.is_some() { style("VERIFIED").green() } else { style("PENDING").yellow() });
            }
        }
        Ok(cliff_watch_core::protocol::Response::Error(e)) => {
            if !short { eprintln!("❌ Daemon error: {}", e); }
            process::exit(1);
        }
        Ok(_) => {
            if !short { eprintln!("❌ Unexpected response from daemon"); }
            process::exit(1);
        }
        Err(e) => {
            if !short { eprintln!("❌ Could not connect to daemon: {}. Is it running?", e); }
            process::exit(1);
        }
    }
}

/// Query the daemon for information
pub async fn query_daemon(request: cliff_watch_core::protocol::Request) -> anyhow::Result<cliff_watch_core::protocol::Response> {
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
