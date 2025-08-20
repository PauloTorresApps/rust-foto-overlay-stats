// ============================================================================
// src/parsers/fit.rs - Parser para arquivos FIT
// ============================================================================

use fitparser::{FitDataRecord, Value};
use std::fs;
use std::path::PathBuf;
use crate::error::{AppResult, AppError};
use super::ActivityData;

/// Faz o parsing de um arquivo FIT e retorna os dados da atividade
pub fn parse_fit(path: &PathBuf) -> AppResult<ActivityData> {
    println!("Lendo arquivo FIT: {:?}", path);
    
    let data = fs::read(path)?;
    let mut session_data: Option<FitDataRecord> = None;
    let mut device_name = "Dispositivo desconhecido".to_string();
    let mut records: Vec<FitDataRecord> = Vec::new();

    // Coleta todos os registros
    for record in fitparser::from_bytes(&data)
        .map_err(|e| AppError::ParseError(format!("Erro ao ler arquivo FIT: {}", e)))? {
        records.push(record);
    }

    // Processa os registros para encontrar Session e DeviceInfo
    for record in records {
        match record.kind() {
            fitparser::profile::MesgNum::Session => {
                session_data = Some(record);
            }
            fitparser::profile::MesgNum::DeviceInfo => {
                for field in record.fields() {
                    if field.name() == "product_name" {
                        if let Value::String(name) = field.value() {
                            device_name = name.clone();
                        }
                    }
                }
            }
            _ => {} // Ignora outros tipos de mensagem
        }
    }

    // Verifica se encontrou dados de sessão
    let session = session_data.ok_or_else(|| 
        AppError::ParseError("Dados de sessão não encontrados no arquivo FIT".to_string()))?;

    // Função auxiliar para extrair campos da sessão
    let get_field = |name: &str| -> Option<Value> {
        session.fields().iter()
            .find(|f| f.name() == name)
            .map(|f| f.value().clone())
    };

    // Extrai o timestamp de início
    let start_time = match get_field("start_time") {
        Some(Value::Timestamp(dt)) => dt,
        _ => return Err(AppError::ParseError(
            "Campo 'start_time' não encontrado no arquivo FIT".to_string())),
    };

    // Constrói e retorna os dados da atividade
    Ok(ActivityData {
        total_time_seconds: get_field("total_elapsed_time").map_or(0.0, |v| match v {
            Value::Float64(val) => val,
            Value::Float32(val) => val as f64,
            Value::UInt32(val) => val as f64,
            _ => 0.0
        }),
        calories: get_field("total_calories").map_or(0, |v| match v {
            Value::UInt16(val) => val,
            Value::UInt32(val) => val as u16,
            _ => 0
        }),
        avg_hr: get_field("avg_heart_rate").map_or(0, |v| match v {
            Value::UInt8(val) => val,
            Value::UInt16(val) => val as u8,
            _ => 0
        }),
        max_hr: get_field("max_heart_rate").map_or(0, |v| match v {
            Value::UInt8(val) => val,
            Value::UInt16(val) => val as u8,
            _ => 0
        }),
        start_time: start_time.into(),
        device_name,
    })
}