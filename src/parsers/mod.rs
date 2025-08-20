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

    /// Normaliza o nome do dispositivo para formato consistente "Marca Modelo"
    pub fn normalize_device_name(&mut self) {
        self.device_name = Self::normalize_device_name_static(&self.device_name);
    }

    /// Normaliza o nome do dispositivo (versão estática)
    fn normalize_device_name_static(device_name: &str) -> String {
        let name_lower = device_name.to_lowercase();
        
        // Se já contém "garmin" no nome, apenas capitaliza
        if name_lower.contains("garmin") {
            return Self::capitalize_device_name(device_name);
        }
        
        // Lista de modelos Garmin conhecidos para detecção
        let garmin_models = [
            "forerunner", "fenix", "venu", "vivoactive", "instinct",
            "epix", "enduro", "approach", "marq", "lily", "tactix", "descent"
        ];
        
        // Verifica se o nome contém algum modelo Garmin
        for model in &garmin_models {
            if name_lower.contains(model) {
                return format!("Garmin {}", Self::capitalize_device_name(device_name));
            }
        }
        
        // Se for apenas "garmin" ou similar, retorna genérico
        if name_lower == "garmin" || name_lower.starts_with("garmin") {
            return "Garmin Device".to_string();
        }
        
        // Caso contrário, retorna o nome original capitalizado
        Self::capitalize_device_name(device_name)
    }
    
    /// Capitaliza adequadamente o nome do dispositivo
    fn capitalize_device_name(name: &str) -> String {
        name.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}