// ============================================================================
// src/cli.rs - Configuração da interface de linha de comando
// ============================================================================

use clap::Parser;
use std::path::PathBuf;

/// Adiciona um overlay de estatísticas de um arquivo TCX ou FIT a uma imagem.
#[derive(Parser, Debug)]
#[command(
    author = "Paulo Roberto Torres",
    version = "1.0.0",
    about = "TCX/FIT Image Overlay Tool",
    long_about = "Adiciona um overlay de estatísticas de treino de arquivos TCX ou FIT a uma imagem."
)]
pub struct CliArgs {
    /// Caminho para a imagem de entrada
    #[arg(short = 'i', long = "image", value_name = "IMAGEM", help = "Caminho para a imagem")]
    pub image_path: PathBuf,
    
    /// Caminho para o arquivo de atividade (TCX ou FIT)
    #[arg(short = 'f', long = "file", value_name = "ARQUIVO", help = "Caminho para o arquivo TCX ou FIT")]
    pub activity_path: PathBuf,
    
    /// Caminho de saída (posicional - opcional)
    #[arg(help = "Caminho para salvar a imagem final (ex: resultado.png)")]
    pub output_path: Option<PathBuf>,
}