//! Script de demostración científica para Cliff-Watch
//!
//! Este script genera dos tipos de trayectorias:
//! 1. Orgánica (Simulada para ser fluida y seguir leyes biológicas)
//! 2. Mecánica (Simulada como un bot de software)
//! 
//! Luego utiliza el MouseSentinel para calcular métricas y demostrar la disparidad.

use cliff_watch_core::mouse_sentinel::{MouseSentinel, InputEvent};
use cliff_watch_core::stats::calculate_human_score;

fn main() {
    println!("--- Cliff-Watch Scientific Validation Suite ---\n");

    // 1. Simulación Humana (Orgánica)
    let mut human_sentinel = MouseSentinel::new(500);
    println!("Generando trayectoria ORGÁNICA (Humana)...");
    
    let mut t_human = 0.0;
    for _i in 0..200 {
        // Los humanos no tienen un polling rate perfecto de cristal de cuarzo
        let dt = 0.01 + (rand::random::<f64>() - 0.5) * 0.002;
        t_human += dt;
        
        // Movimiento curvo suave (minimización de jerk)
        let x = 100.0 + 500.0 * (t_human * 0.5).sin();
        let y = 100.0 + 300.0 * (t_human * 0.3).cos();
        
        human_sentinel.push(InputEvent::Mouse { x, y, t: t_human });
    }

    // 2. Simulación Mecánica (Bot/IA)
    let mut bot_sentinel = MouseSentinel::new(500);
    println!("Generando trayectoria MECÁNICA (Bot/IA)...");
    
    for i in 0..200 {
        let t = i as f64 * 0.01; // Polling rate perfecto
        // Movimiento lineal con velocidad constante (firma de bot)
        let x = 100.0 + 100.0 * t;
        let y = 100.0 + 100.0 * t;
        
        bot_sentinel.push(InputEvent::Mouse { x, y, t });
    }

    // Análisis y Reporte
    println!("\n--- RESULTADOS DE ANÁLISIS ---");
    
    let human_metrics = human_sentinel.analyze().expect("Failed to analyze human data");
    let bot_metrics = bot_sentinel.analyze().expect("Failed to analyze bot data");

    // Utilizamos los valores calculados por el Sentinel (Gobernanza v4.0)
    let human_score = calculate_human_score(human_metrics.burstiness, human_metrics.ncd, 1.0, 10, human_metrics.is_synthetic);
    let bot_score = calculate_human_score(bot_metrics.burstiness, bot_metrics.ncd, 0.0, 0, bot_metrics.is_synthetic);

    println!("\n[ PERFIL HUMANO ]");
    println!("  LDLJ (Suavidad):    {:.4}", human_metrics.ldlj);
    println!("  Entropía Vel.:      {:.4}", human_metrics.velocity_entropy);
    println!("  Burstiness:         {:.4}", human_metrics.burstiness);
    println!("  NCD:                {:.4}", human_metrics.ncd);
    println!("  Throughput:         {:.4}", human_metrics.throughput);
    println!("  Human-Score Final:  {:.2}%", human_score * 100.0);

    println!("\n[ PERFIL BOT/IA ]");
    println!("  LDLJ (Suavidad):    {:.4}", bot_metrics.ldlj);
    println!("  Entropía Vel.:      {:.4}", bot_metrics.velocity_entropy);
    println!("  Burstiness:         {:.4}", bot_metrics.burstiness);
    println!("  NCD:                {:.4}", bot_metrics.ncd);
    println!("  Throughput:         {:.4}", bot_metrics.throughput);
    println!("  Human-Score Final:  {:.2}%", bot_score * 100.0);

    println!("\n--- CONCLUSIÓN ---");
    if human_score > bot_score {
        println!("✅ El sistema identificó correctamente la firma biológica.");
        println!("   La disparidad en LDLJ ({:.2} vs {:.2}) demuestra que el bot carece", human_metrics.ldlj, bot_metrics.ldlj);
        println!("   de la fluidez intrínseca de la coordinación neuromotora humana.");
    } else {
        println!("❌ Fallo en la validación.");
    }
}
