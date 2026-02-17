//! Mouse Sentinel - Monitor de movimiento de mouse para análisis cinemático
//!
//! Este módulo implementa un sistema de monitoreo de eventos de mouse que calcula
//! métricas cinemáticas para análisis de comportamiento humano.
//!
//! # Responsabilidades
//! - Captura y almacenamiento de eventos de mouse (posición + timestamp)
//! - Cálculo de métricas cinemáticas (velocidad, aceleración, jerk)
//! - Análisis de entropía de movimiento
//! - Detección de patrones de movimiento anómalos
//!
//! # Ejemplo de uso
//! ```
//! use cliff_watch_core::mouse_sentinel::{MouseSentinel, InputEvent};
//!
//! let mut sentinel = MouseSentinel::new(100); // Buffer para 100 eventos
//! 
//! // Capturar eventos de mouse
//! sentinel.push(InputEvent::Mouse {
//!     x: 100.0,
//!     y: 200.0,
//!     t: 123456.0, // timestamp en segundos
//! });
//!
//! // Analizar patrones de movimiento
//! // (requiere al menos 4 eventos para análisis)
//! // for i in 0..5 { sentinel.capture_event(i as f64, i as f64); }
//! // let metrics = sentinel.analyze();
//! ```

use std::collections::VecDeque;
use thiserror::Error;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum InputEvent {
    Mouse { x: f64, y: f64, t: f64 },
    Keyboard { t: f64 },
}

#[derive(Debug, Error)]
pub enum MouseSentinelError {
    #[error("Insufficient data points for analysis")]
    InsufficientData,
    #[error("Invalid timeline (non-monotonic timestamps)")]
    InvalidTimeline,
    #[error("Degenerate motion (zero velocity)")]
    DegenerateMotion,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KinematicMetrics {
    /// Log-Dimensional Less Jerk - métrica de suavidad de movimiento
    pub ldlj: f64,
    /// Entropía de la distribución de velocidades
    pub velocity_entropy: f64,
    /// Entropía de la curvatura de la trayectoria
    pub curvature_entropy: f64,
    /// Throughput - distancia total recorrida por unidad de tiempo
    pub throughput: f64,
    /// Burstiness - variabilidad en los intervalos entre eventos
    pub burstiness: f64,
    /// NCD - medida de complejidad algorítmica/compresibilidad
    pub ncd: f64,
    /// Flag de detección de actividad sintética (bot/script)
    pub is_synthetic: bool,
}

#[derive(Debug)]
pub struct MouseSentinel {
    pub buffer: VecDeque<InputEvent>, // Guardaremos solo los eventos de mouse aquí para análisis cinemático
    pub capacity: usize,
}

impl MouseSentinel {
    /// Crea un nuevo MouseSentinel con capacidad de buffer especificada
    ///
    /// # Argumentos
    /// * `capacity` - Número máximo de eventos a almacenar
    ///
    /// # Ejemplo
    /// ```
    /// use cliff_watch_core::mouse_sentinel::MouseSentinel;
    /// let sentinel = MouseSentinel::new(100);
    /// ```
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Agrega un evento de mouse al buffer
    ///
    /// # Argumentos
    /// * `event` - Evento de mouse a almacenar
    ///
    /// # Comportamiento
    /// Si el buffer está lleno, se elimina el evento más antiguo
    ///
    /// # Ejemplo
    /// ```
    /// use cliff_watch_core::mouse_sentinel::{MouseSentinel, InputEvent};
    /// let mut sentinel = MouseSentinel::new(10);
    /// sentinel.push(InputEvent::Mouse { x: 100.0, y: 200.0, t: 123.0 });
    /// ```
    pub fn push(&mut self, event: InputEvent) {
        if let InputEvent::Mouse { .. } = event {
            if self.buffer.len() == self.capacity {
                self.buffer.pop_front();
            }
            self.buffer.push_back(event);
        }
        // Los eventos de teclado se manejan a otro nivel (ritmo), 
        // no entran en el análisis cinemático de trayectorias.
    }

    /// Método alternativo para capturar eventos con timestamp automático
    ///
    /// # Argumentos
    /// * `x` - Coordenada X del evento
    /// * `y` - Coordenada Y del evento
    ///
    /// # Ejemplo
    /// ```
    /// use cliff_watch_core::mouse_sentinel::MouseSentinel;
    /// let mut sentinel = MouseSentinel::new(10);
    /// sentinel.capture_event(100.0, 200.0);
    /// ```
    pub fn capture_event(&mut self, x: f64, y: f64) {
        let event = InputEvent::Mouse {
            x,
            y,
            t: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("System time is before UNIX_EPOCH")
                .as_secs_f64(),
        };
        self.push(event);
    }

    /// Analiza los eventos capturados y calcula métricas cinemáticas
    ///
    /// # Retorna
    /// * `Ok(KinematicMetrics)` - Métricas calculadas exitosamente
    /// * `Err(MouseSentinelError)` - Error si no hay suficientes datos o datos inválidos
    ///
    /// # Requisitos
    /// - Al menos 4 eventos en el buffer
    /// - Timeline monótono (timestamps en orden creciente)
    /// - Movimiento no degenerado (velocidad no cero)
    ///
    /// # Ejemplo
    /// ```
    /// use cliff_watch_core::mouse_sentinel::MouseSentinel;
    /// let mut sentinel = MouseSentinel::new(10);
    /// // Añadir puntos ficticios para el test
    /// for i in 0..5 { sentinel.capture_event(i as f64, i as f64); }
    /// match sentinel.analyze() {
    ///     Ok(metrics) => println!("LDLJ: {}", metrics.ldlj),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn analyze(&self) -> Result<KinematicMetrics, MouseSentinelError> {
        if self.buffer.len() < 4 {
            return Err(MouseSentinelError::InsufficientData);
        }

        let v = velocities(&self.buffer)?;
        let a = accelerations(&v)?;
        let j = jerks(&a)?;

        let t_start = match self.buffer.front() {
            Some(event) => match event {
                InputEvent::Mouse { t, .. } => *t,
                _ => return Err(MouseSentinelError::InsufficientData),
            },
            None => return Err(MouseSentinelError::InsufficientData),
        };
        let t_end = match self.buffer.back() {
            Some(event) => match event {
                InputEvent::Mouse { t, .. } => *t,
                _ => return Err(MouseSentinelError::InsufficientData),
            },
            None => return Err(MouseSentinelError::InsufficientData),
        };

        let duration = t_end - t_start;
        if duration <= 0.0 {
            return Err(MouseSentinelError::InvalidTimeline);
        }

        let peak_v = v.iter().map(|x| x.1).fold(0.0, f64::max);
        if peak_v == 0.0 {
            return Err(MouseSentinelError::DegenerateMotion);
        }

        let ldlj = ldlj(&j, duration, peak_v)?;
        let velocity_entropy = shannon_entropy(v.iter().map(|(_, v)| *v))?;
        let curvature_entropy = curvature_entropy(&self.buffer)?;
        let throughput = path_length(&self.buffer)? / duration;

        // Nuevas métricas científicas
        let timestamps: Vec<f64> = self.buffer.iter().filter_map(|e| match e {
            InputEvent::Mouse { t, .. } => Some(*t),
            _ => None,
        }).collect();
        let burstiness = crate::stats::calculate_burstiness(&timestamps);
        let is_synthetic = crate::stats::is_synthetic_pattern(&timestamps);

        // Para NCD usamos la serie de velocidades como firma de comportamiento
        let v_bytes: Vec<u8> = v.iter()
            .flat_map(|(_, vel)| vel.to_le_bytes().to_vec())
            .collect();
        // Comparamos con una secuencia monótona del mismo tamaño para medir "sorpresa"
        let reference = vec![0u8; v_bytes.len()];
        let ncd = crate::stats::calculate_ncd(&v_bytes, &reference);

        Ok(KinematicMetrics {
            ldlj,
            velocity_entropy,
            curvature_entropy,
            throughput,
            burstiness,
            ncd,
            is_synthetic,
        })
    }
}

/* ---------------- Funciones internas de cálculo ---------------- */

/// Calcula velocidades entre eventos consecutivos
fn velocities(events: &VecDeque<InputEvent>) -> Result<Vec<(f64, f64)>, MouseSentinelError> {
    let mut out = Vec::new();
    for w in events.as_slices().0.windows(2) {
        let (t0, x0, y0) = match w[0] {
            InputEvent::Mouse { t, x, y } => (t, x, y),
            _ => continue,
        };
        let (t1, x1, y1) = match w[1] {
            InputEvent::Mouse { t, x, y } => (t, x, y),
            _ => continue,
        };

        let dt = t1 - t0;
        if dt <= 0.0 {
            return Err(MouseSentinelError::InvalidTimeline);
        }
        let dx = x1 - x0;
        let dy = y1 - y0;
        let v = (dx * dx + dy * dy).sqrt() / dt;
        out.push((t1, v));
    }
    Ok(out)
}

/// Calcula aceleraciones a partir de velocidades
fn accelerations(v: &[(f64, f64)]) -> Result<Vec<(f64, f64)>, MouseSentinelError> {
    let mut out = Vec::new();
    for w in v.windows(2) {
        let dt = w[1].0 - w[0].0;
        if dt <= 0.0 {
            return Err(MouseSentinelError::InvalidTimeline);
        }
        out.push((w[1].0, (w[1].1 - w[0].1) / dt));
    }
    Ok(out)
}

/// Calcula jerks (derivada de la aceleración) a partir de aceleraciones
fn jerks(a: &[(f64, f64)]) -> Result<Vec<(f64, f64)>, MouseSentinelError> {
    let mut out = Vec::new();
    for w in a.windows(2) {
        let dt = w[1].0 - w[0].0;
        if dt <= 0.0 {
            return Err(MouseSentinelError::InvalidTimeline);
        }
        out.push((w[1].0, (w[1].1 - w[0].1) / dt));
    }
    Ok(out)
}

/// Calcula la métrica LDLJ (Log-Dimensional Less Jerk)
fn ldlj(j: &[(f64, f64)], duration: f64, peak_v: f64) -> Result<f64, MouseSentinelError> {
    let integral: f64 = j
        .windows(2)
        .map(|w| {
            let dt = w[1].0 - w[0].0;
            w[0].1.powi(2) * dt
        })
        .sum();

    if integral == 0.0 {
        return Ok(0.0);
    }

    Ok(-((integral * duration.powi(3)) / peak_v.powi(2)).ln())
}

/// Calcula la entropía de Shannon para una distribución de valores
fn shannon_entropy<I: Iterator<Item = f64>>(values: I) -> Result<f64, MouseSentinelError> {
    let mut counts = std::collections::HashMap::new();
    let mut total = 0.0;

    for value in values {
        let rounded_value = (value * 1000.0).round() as i64;
        *counts.entry(rounded_value).or_insert(0.0) += 1.0;
        total += 1.0;
    }

    if total == 0.0 {
        return Ok(0.0);
    }

    let entropy = counts.values().map(|&count| {
        let p: f64 = count / total;
        if p > 0.0 {
            p * p.log2()
        } else {
            0.0
        }
    }).sum::<f64>();

    Ok(-entropy)
}

/// Calcula la entropía de curvatura de la trayectoria
fn curvature_entropy(events: &VecDeque<InputEvent>) -> Result<f64, MouseSentinelError> {
    let mut curvatures = Vec::new();

    for w in events.as_slices().0.windows(3) {
        let (x0, y0) = match w[0] { InputEvent::Mouse { x, y, .. } => (x, y), _ => continue };
        let (x1, y1) = match w[1] { InputEvent::Mouse { x, y, .. } => (x, y), _ => continue };
        let (x2, y2) = match w[2] { InputEvent::Mouse { x, y, .. } => (x, y), _ => continue };

        let dx1 = x1 - x0;
        let dy1 = y1 - y0;
        let dx2 = x2 - x1;
        let dy2 = y2 - y1;

        let cross = dx1 * dy2 - dy1 * dx2;
        let dot1 = dx1 * dx1 + dy1 * dy1;
        let dot2 = dx2 * dx2 + dy2 * dy2;

        if dot1 > 0.0 && dot2 > 0.0 {
            let curvature = cross.abs() / (dot1.sqrt() * dot2.sqrt() + 1e-10);
            curvatures.push(curvature);
        }
    }

    shannon_entropy(curvatures.into_iter())
}

/// Calcula la longitud total de la trayectoria
fn path_length(events: &VecDeque<InputEvent>) -> Result<f64, MouseSentinelError> {
    let mut length = 0.0;

    for w in events.as_slices().0.windows(2) {
        let (x0, y0) = match w[0] { InputEvent::Mouse { x, y, .. } => (x, y), _ => continue };
        let (x1, y1) = match w[1] { InputEvent::Mouse { x, y, .. } => (x, y), _ => continue };

        let dx = x1 - x0;
        let dy = y1 - y0;
        length += (dx * dx + dy * dy).sqrt();
    }

    Ok(length)
}
