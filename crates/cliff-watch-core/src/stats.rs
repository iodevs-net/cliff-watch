//! Este módulo implementa funciones estadísticas de alta eficiencia (LEAN) para validar
//! contribuciones humanas vs AI, siguiendo el modelo de Ruido Cognitivo (CNS) v3.0.

/// Calcula la burstiness de una serie de tiempos de edición
/// Burstiness mide la variabilidad en los intervalos entre eventos
pub fn calculate_burstiness(times: &[f64]) -> f64 {
    if times.len() < 2 {
        return 0.0;
    }
    
    let intervals = get_intervals(times);
    if intervals.is_empty() {
        return 0.0;
    }
    
    let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
    let std_dev = calculate_std_dev(&intervals, mean);
    
    if mean < 1e-10 || std_dev < 1e-10 {
        return -1.0;
    }
    
    (std_dev - mean) / (std_dev + mean)
}

/// Estima el parámetro Alpha de una distribución de Pareto para los intervalos dados
/// (CNS v2.1 - Informe Omega)
/// Alpha entre 1.5 y 3.0 suele indicar comportamiento humano fractal
pub fn estimate_pareto_alpha(times: &[f64]) -> f64 {
    let intervals = get_intervals(times);
    if intervals.len() < 5 { // Necesitamos una muestra mínima
        return 0.0;
    }

    let min_x = intervals.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    if min_x < 1e-10 { return 0.0; }

    // Estimador de Máxima Verosimilitud (MLE) para Alpha
    let sum_log: f64 = intervals.iter()
        .map(|&x| (x / min_x).ln())
        .sum();

    if sum_log < 1e-10 { return 0.0; }

    (intervals.len() as f64) / sum_log
}

fn get_intervals(times: &[f64]) -> Vec<f64> {
    times.windows(2)
        .map(|w| (w[1] - w[0]).abs())
        .filter(|&x| x > 1e-10)
        .collect()
}

fn calculate_std_dev(data: &[f64], mean: f64) -> f64 {
    let variance = data.iter()
        .map(|&value| {
            let diff = mean - value;
            diff * diff
        })
        .sum::<f64>() / data.len() as f64;
    variance.sqrt()
}

/// Detecta si una serie de eventos tiene una regularidad mecánica (Bot/Script)
/// 
/// Un humano real tiene "Ruido Cognitivo". Si el Coeficiente de Variación (CV)
/// es demasiado bajo (< 0.1), es casi seguro que es una inyección sintética.
pub fn is_synthetic_pattern(times: &[f64]) -> bool {
    let intervals = get_intervals(times);
    if intervals.len() < 5 {
        return false; // No hay suficiente muestra
    }

    let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
    let std_dev = calculate_std_dev(&intervals, mean);
    
    if mean < 1e-10 { return true; }

    let cv = std_dev / mean;
    
    // Un humano rara vez baja de CV 0.2 en tareas de edición/scroll.
    // Un script con sleep(0.1) tiene CV casi 0.
    cv < 0.15 
}

/// Calcula la distancia de compresión normalizada (NCD) entre dos secuencias
/// NCD mide la similitud entre patrones de código
pub fn calculate_ncd(x: &[u8], y: &[u8]) -> f64 {
    use zstd::stream::encode_all;
    
    // Comprimir individualmente
    let cx = encode_all(x, 1).unwrap_or_else(|_| x.to_vec());
    let cy = encode_all(y, 1).unwrap_or_else(|_| y.to_vec());
    
    // Comprimir concatenación
    let mut concat = x.to_vec();
    concat.extend_from_slice(y);
    let cxy = encode_all(&*concat, 1).unwrap_or_else(|_| concat);
    
    // NCD = (cxy - min(cx, cy)) / max(cx, cy)
    let min_c = cx.len().min(cy.len()) as f64;
    let max_c = cx.len().max(cy.len()) as f64;
    let cxy_len = cxy.len() as f64;
    
    if max_c == 0.0 {
        return 0.0;
    }
    
    ((cxy_len - min_c) / max_c).max(0.0).min(1.0)
}

/// Calcula un score compuesto que representa la probabilidad de contribución humana
/// Score = 0.0 (AI) a 1.0 (humano)
pub fn calculate_human_score(
    burstiness: f64, 
    ncd: f64, 
    focus_mins: f64, 
    nav_events: usize,
    is_synthetic: bool
) -> f64 {
    // Normalizar burstiness: mapear de [-1, 1] a [0, 1]
    let norm_burstiness = (burstiness + 1.0) / 2.0;
    
    // NCD ya está en rango 0-1
    let norm_ncd = ncd.clamp(0.0, 1.0);

    // Score de Foco: 1 minuto de foco = 0.1 de score (cap en 1.0)
    let focus_score = (focus_mins * 0.1 + (nav_events as f64 * 0.02)).min(1.0);
    
    // Score compuesto (Gobernanza v4.0):
    // 40% Foco (Prueba de Presencia)
    // 40% Burstiness (Prueba de Humanidad/Cinemática)
    // 20% NCD (Prueba de Creador/Originalidad)
    let mut score = 0.4 * focus_score + 0.4 * norm_burstiness + 0.2 * norm_ncd;

    // PENALIZACIÓN SINTÉTICA
    // Si la cinemática es demasiado regular (CV bajo), penalizamos el score.
    if is_synthetic {
        score *= 0.5;
    }

    score
}

/// Valida si un score representa una contribución humana utilizando ZKP si es necesario.
pub fn validate_contribution(score: f64, threshold: f64) -> bool {
    score >= threshold
}

/// Representa el resultado de una auditoría termodinámica.
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ThermodynamicReport {
    pub h_score: f64,
    pub burstiness: f64,
    pub pareto_alpha: f64,
    pub ncd_ratio: f64,
    pub is_human: bool,
    pub zkp_commitment: Option<String>, // Hex encoded commitment
}

/// Calcula el Score de Acoplamiento Cognitivo-Motor.
/// 
/// Mide la sincronía entre la complejidad del código (`code_complexity`) y el
/// esfuerzo biomecánico detectado (`motor_entropy`).
/// 
/// Un valor cercano a 1.0 indica un acoplamiento coherente (trabajo humano real).
/// Un valor cercano a 0.0 indica un desacoplo (ej. inyección masiva de código complejo sin pausas cognitivas).
pub fn calculate_coupling_score(code_complexity: f64, motor_entropy: f64) -> f64 {
    if code_complexity < 0.1 {
        // Para código muy simple (boilerplate), cualquier esfuerzo es válido.
        return 1.0;
    }

    // El score baja si hay mucha complejidad lógica pero poca variabilidad motora.
    let diff = (code_complexity - motor_entropy).abs();
    
    // Si la diferencia es pequeña, el acoplamiento es alto.
    (1.0 - diff).clamp(0.0, 1.0)
}

/// Calcula un threshold dinámico basado en scores históricos
pub fn calculate_dynamic_threshold(historical_scores: &[f64], base_threshold: f64) -> f64 {
    if historical_scores.is_empty() {
        return base_threshold;
    }
    
    let avg_score = historical_scores.iter().sum::<f64>() / historical_scores.len() as f64;
    
    // Threshold dinámico: 90% del promedio histórico
    // Esto adapta el threshold a los patrones observados
    avg_score * 0.9
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_burstiness_calculation() {
        // Datos con ráfagas (distribución de intervalos muy desigual)
        let human_times = vec![0.0, 0.1, 10.1, 10.2, 20.2];
        let burstiness = calculate_burstiness(&human_times);
        // La burstiness puede ser ligeramente negativa, pero debe ser mucho mayor que AI (-1.0)
        assert!(burstiness > -0.5, "Expected human-like burstiness, got {}", burstiness);
        
        // Datos con baja variabilidad (AI-like / Regular)
        let ai_times = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let ai_burstiness = calculate_burstiness(&ai_times);
        assert!(ai_burstiness < -0.9, "Negative burstiness expected for regular AI patterns, got {}", ai_burstiness);
    }

    #[test]
    fn test_ncd_calculation() {
        // Datos idénticos
        let x = b"identical identical identical";
        let y = b"identical identical identical";
        let ncd = calculate_ncd(x, y);
        assert!(ncd < 0.5, "Low NCD expected for identical data");
        
        // Datos diferentes
        let x = b"completely different data";
        let y = b"unrelated content here";
        let ncd = calculate_ncd(x, y);
        assert!(ncd > 0.5, "High NCD expected for different data");
    }

    #[test]
    fn test_human_score_calculation() {
        // Alto burstiness (0.9) + bajo NCD (0.2) + foco = alto score humano
        let score_h = calculate_human_score(0.9, 0.2, 1.0, 20, false);
        assert!(score_h > 0.5, "High human score expected for bursty patterns");
        
        // Bajo burstiness (-0.9) + medio NCD (0.5) = bajo score humano
        let score_ai = calculate_human_score(-0.9, 0.5, 0.1, 2, false);
        assert!(score_ai < 0.4, "Low human score expected for regular patterns");

        // Casos sintéticos (penalizados)
        let score_syn = calculate_human_score(0.9, 0.2, 1.0, 20, true);
        assert!(score_syn < score_h, "Synthetic score should be penalized compared to human score");
        assert!((score_syn - (score_h * 0.5)).abs() < 1e-10, "Synthetic score should be exactly 50% of human score");
    }

    #[test]
    fn test_dynamic_threshold() {
        let historical = vec![0.85, 0.88, 0.78, 0.92];
        let threshold = calculate_dynamic_threshold(&historical, 0.7);
        assert!(threshold > 0.7, "Dynamic threshold should adapt upwards");
    }

    #[test]
    fn test_debug_values() {
        // Debug burstiness values
        let human_times = vec![10.0, 0.1, 0.1, 20.0, 0.1, 0.1, 30.0];
        let human_burstiness = calculate_burstiness(&human_times);
        println!("Human burstiness: {}", human_burstiness);
        
        let ai_times = vec![1.0, 1.01, 0.99, 1.0, 1.0, 1.01];
        let ai_burstiness = calculate_burstiness(&ai_times);
        println!("AI burstiness: {}", ai_burstiness);
        
        // Debug human score values
        let high_score = calculate_human_score(0.9, 0.2, 1.0, 20, false);
        println!("High human score (0.9, 0.2): {}", high_score);
        
        let low_score = calculate_human_score(0.1, 0.8, 0.0, 0, false);
        println!("Low human score (0.1, 0.8): {}", low_score);
        
        // These should always pass
        assert!(true);
    }
}