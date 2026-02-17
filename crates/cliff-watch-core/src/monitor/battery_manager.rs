//! Attention battery management for cognitive effort tracking.

use std::time::{Duration, SystemTime};

/// Batería de Atención (Batería Kinética/Foco)
///
/// Acumula "energía" basada en:
/// - **v1.0 (Legacy)**: Complejidad del movimiento humano (mouse/teclado)
/// - **v2.0 (Focus)**: Tiempo de foco activo + ráfagas de edición
///
/// La energía se pierde con el tiempo (leaky bucket). Representa el esfuerzo
/// cognitivo disponible para validar código.
#[derive(Debug, Clone)]
pub struct AttentionBattery {
    pub level: f64,
    pub capacity: f64,
    pub last_decay: SystemTime,
    pub leak_rate: f64,
    pub causal_event_count: usize, // Conteo de eventos reales procesados
}

impl Default for AttentionBattery {
    fn default() -> Self {
        Self {
            level: 0.0,
            capacity: 100.0,
            last_decay: SystemTime::now(),
            leak_rate: 0.5,
            causal_event_count: 0,
        }
    }
}

impl AttentionBattery {
    pub fn new() -> Self {
        Self::default()
    }

    /// [v1.0 LEGACY] Carga basándose en entropía motora y eventos de hardware
    ///
    /// Este método se mantiene para compatibilidad con el backend `evdev`.
    /// Para v2.0, usar `charge_focus()` en su lugar.
    pub fn charge(&mut self, motor_entropy: f64, duration: Duration, hardware_events: usize, keyboard_hits: usize) {
        self.apply_decay();
        
        // [CAUSALIDAD] Validamos que han ocurrido eventos reales
        let events_delta = hardware_events.saturating_sub(self.causal_event_count);
        if events_delta == 0 {
            return; 
        }

        // La carga es proporcional a la entropía motora Y al volumen de teclado
        // El teclado es un indicador de "trabajo duro" (sweat).
        let mouse_charge = (motor_entropy * duration.as_secs_f64() * 5.0).min(events_delta as f64 * 0.1);
        let keyboard_charge = (keyboard_hits as f64 * 0.5).min(20.0); // Cap de carga por intervalo de teclado
        
        self.level = (self.level + mouse_charge + keyboard_charge).min(self.capacity);
        self.causal_event_count = hardware_events;
    }

    /// [v2.0 FOCUS] Carga basándose en tiempo de foco activo
    ///
    /// Este método implementa el modelo "Proof of Focus" que valida
    /// el trabajo cognitivo (Deep Work) sin requerir movimiento constante.
    ///
    /// ## Fórmula de Carga
    /// - Tiempo de foco: 1 minuto de foco activo = 5 puntos de energía
    /// - Ediciones: sqrt(ráfagas) * 2 puntos (recompensa incremental decreciente)
    /// - Navegación: 0.5 puntos por evento (lectura activa)
    ///
    /// ## Ejemplo
    /// Un Senior que lee documentación por 10 minutos (foco activo)
    /// y luego hace 4 ediciones rápidas obtiene: 50 + 4 = 54 puntos.
    pub fn charge_focus(&mut self, focus_duration: Duration, edit_burst_count: usize, navigation_events: usize) {
        self.apply_decay();
        
        // Tiempo de foco activo carga linealmente (1 min focus = 10 pts)
        let focus_charge = (focus_duration.as_secs_f64() / 60.0) * 10.0;
        
        // Bonus por ediciones reales (prueba de interacción)
        // Usamos sqrt para recompensa incremental decreciente
        let edit_bonus = (edit_burst_count as f64).sqrt() * 5.0;
        
        // Bonus por navegación (lectura activa: scroll, goto definition, etc.)
        let nav_bonus = (navigation_events as f64) * 2.0;
        
        let total_charge = focus_charge + edit_bonus + nav_bonus;
        self.level = (self.level + total_charge).min(self.capacity);
    }

    /// Consume energía (Costo Entrópico)
    pub fn consume(&mut self, cost: f64) -> bool {
        self.apply_decay();
        if self.level >= cost {
            self.level -= cost;
            true
        } else {
            false
        }
    }

    fn apply_decay(&mut self) {
        let now = SystemTime::now();
        if let Ok(elapsed) = now.duration_since(self.last_decay) {
            let decay = elapsed.as_secs_f64() * self.leak_rate;
            self.level = (self.level - decay).max(0.0);
            self.last_decay = now;
        }
    }
}
