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
    
    // Se o usuário especificou uma saída personalizada, usa ela
    if let Some(custom_output) = args.output_path {
        println!("📌 Usando saída personalizada: {:?}", custom_output);
        processor.set_output_path(custom_output);
    }
    // Caso contrário, usa a saída automática já configurada
    
    processor.process_activity_file(&args.activity_path)?;
    processor.save_result()?;
    
    println!("✅ Processo concluído com sucesso!");
    Ok(())
}