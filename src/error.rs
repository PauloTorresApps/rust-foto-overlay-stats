// ============================================================================
// src/error.rs - Gerenciamento centralizado de erros
// ============================================================================

use std::fmt;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    IoError(std::io::Error),
    ImageError(image::ImageError),
    ParseError(String),
    FontError(String),
    InvalidFormat(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::IoError(e) => write!(f, "Erro de E/S: {}", e),
            AppError::ImageError(e) => write!(f, "Erro de imagem: {}", e),
            AppError::ParseError(e) => write!(f, "Erro de parsing: {}", e),
            AppError::FontError(e) => write!(f, "Erro de fonte: {}", e),
            AppError::InvalidFormat(e) => write!(f, "Formato inválido: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::IoError(error)
    }
}

impl From<image::ImageError> for AppError {
    fn from(error: image::ImageError) -> Self {
        AppError::ImageError(error)
    }
}

impl From<Box<dyn std::error::Error>> for AppError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        AppError::ParseError(error.to_string())
    }
}