// ============================================================================
// src/parsers/tcx.rs - Parser para arquivos TCX
// ============================================================================

use serde::Deserialize;
use chrono::{DateTime, Utc};
use std::fs;
use std::path::PathBuf;
use crate::error::{AppResult, AppError};
use super::ActivityData;

#[derive(Debug, Deserialize)]
#[serde(rename = "TrainingCenterDatabase")]
struct TcxDatabase {
    #[serde(rename = "Activities")]
    activities: TcxActivities,
}

#[derive(Debug, Deserialize)]
struct TcxActivities {
    #[serde(rename = "Activity")]
    activity: TcxActivity,
}

#[derive(Debug, Deserialize)]
struct TcxActivity {
    #[serde(rename = "Lap")]
    lap: TcxLap,
    #[serde(rename = "Creator")]
    creator: TcxCreator,
}

#[derive(Debug, Deserialize)]
struct TcxLap {
    #[serde(rename = "TotalTimeSeconds")]
    total_time_seconds: f64,
    #[serde(rename = "Calories")]
    calories: u16,
    #[serde(rename = "AverageHeartRateBpm")]
    avg_hr: TcxHeartRate,
    #[serde(rename = "MaximumHeartRateBpm")]
    max_hr: TcxHeartRate,
    #[serde(rename = "@StartTime")]
    start_time: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct TcxHeartRate {
    #[serde(rename = "Value")]
    value: u8,
}

#[derive(Debug, Deserialize)]
struct TcxCreator {
    #[serde(rename = "Name")]
    name: String,
}

/// Faz o parsing de um arquivo TCX e retorna os dados da atividade
pub fn parse_tcx(path: &PathBuf) -> AppResult<ActivityData> {
    println!("Lendo arquivo TCX: {:?}", path);
    
    let tcx_content = fs::read_to_string(path)?;
    let tcx_data: TcxDatabase = quick_xml::de::from_str(&tcx_content)
        .map_err(|e| AppError::ParseError(format!("Erro ao parsear TCX: {}", e)))?;

    let lap = &tcx_data.activities.activity.lap;
    let creator = &tcx_data.activities.activity.creator;

    Ok(ActivityData {
        total_time_seconds: lap.total_time_seconds,
        calories: lap.calories,
        avg_hr: lap.avg_hr.value,
        max_hr: lap.max_hr.value,
        start_time: lap.start_time,
        device_name: creator.name.clone(),
    })
}