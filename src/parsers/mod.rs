// ============================================================================
// src/parsers/mod.rs - Módulo de parsers
// ============================================================================

pub mod tcx;
pub mod fit;

use chrono::{DateTime, Utc};

/// Estrutura unificada para dados de atividade
#[derive(Debug, Clone)]
pub struct ActivityData {
    pub total_time_seconds: f64,
    pub calories: u16,
    pub avg_hr: u8,
    pub max_hr: u8,
    pub start_time: DateTime<Utc>,
    pub device_name: String,
}

impl ActivityData {
    /// Formata a duração da atividade em formato legível
    pub fn format_duration(&self) -> String {
        let total_seconds = self.total_time_seconds as u32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        format!("{}h {:02}m", hours, minutes)
    }
}