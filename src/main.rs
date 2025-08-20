// ============================================================================
// src/main.rs - Ponto de entrada da aplicação
// ============================================================================

mod cli;
mod parsers;
mod image_processor;
mod error;
mod constants;

use crate::cli::CliArgs;
use crate::image_processor::ImageProcessor;
use crate::error::AppResult;
use clap::Parser;

fn main() -> AppResult<()> {
    let args = CliArgs::parse();
    
    println!("=== TCX/FIT Image Overlay Tool ===");
    
    let mut processor = ImageProcessor::new(&args.image_path)?;
    
    // Define caminho de saída se fornecido
    if let Some(output_path) = args.output_path {
        processor.set_output_path(output_path);
    }
    
    processor.process_activity_file(&args.activity_path)?;
    processor.save_result()?;
    
    println!("Processo concluído com sucesso!");
    Ok(())
}