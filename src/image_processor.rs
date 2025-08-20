// ============================================================================
// src/image_processor.rs - Processamento de imagens e overlay
// ============================================================================

use image::{Rgba, RgbaImage, GenericImageView, imageops};
use imageproc::drawing::{draw_text_mut, text_size};
use rusttype::{Font, Scale};
use chrono::Local;
use std::fs;
use std::path::PathBuf;

use crate::constants::*;
use crate::error::{AppResult, AppError};
use crate::parsers::{ActivityData, tcx, fit};

/// Processador principal de imagens com overlay de estatísticas
pub struct ImageProcessor {
    image: RgbaImage,
    width: u32,
    height: u32,
    font: Font<'static>,
    icon_font: Font<'static>,
    output_path: PathBuf,
}

/// Layout para posicionamento do overlay
struct OverlayLayout {
    max_line_width: i32,
    icon_padding: i32,
    text_line_height: u32,
    total_text_height: u32,
    block_x_start: i32,
    block_y_start: i32,
}

impl ImageProcessor {
    /// Cria um novo processador de imagem
    pub fn new(image_path: &PathBuf) -> AppResult<Self> {
        println!("Carregando imagem: {:?}", image_path);
        let image = image::open(image_path)?.to_rgba8();
        let (width, height) = image.dimensions();

        println!("Carregando fontes...");
        let font = Self::load_font(FONT_PATH)?;
        let icon_font = Self::load_font(ICON_FONT_PATH)?;

        Ok(Self {
            image,
            width,
            height,
            font,
            icon_font,
            output_path: PathBuf::from(DEFAULT_OUTPUT_PATH),
        })
    }

    /// Carrega uma fonte a partir do caminho especificado
    fn load_font(path: &str) -> AppResult<Font<'static>> {
        let font_data = fs::read(path)?;
        Font::try_from_vec(font_data)
            .ok_or_else(|| AppError::FontError(format!("Falha ao carregar fonte: {}", path)))
    }

    /// Define o caminho de saída personalizado
    pub fn set_output_path(&mut self, path: PathBuf) {
        // Se o caminho for um diretório, gera um nome de arquivo padrão
        if path.is_dir() {
            self.output_path = path.join(DEFAULT_OUTPUT_PATH);
        } else {
            self.output_path = path;
        }
    }

    /// Processa um arquivo de atividade e adiciona o overlay à imagem
    pub fn process_activity_file(&mut self, activity_path: &PathBuf) -> AppResult<()> {
        let activity_data = self.parse_activity_file(activity_path)?;
        self.add_overlay(&activity_data)?;
        Ok(())
    }

    /// Determina o tipo de arquivo e chama o parser apropriado
    fn parse_activity_file(&self, path: &PathBuf) -> AppResult<ActivityData> {
        match path.extension().and_then(|s| s.to_str()) {
            Some("tcx") => tcx::parse_tcx(path),
            Some("fit") => fit::parse_fit(path),
            _ => Err(AppError::InvalidFormat(
                "Formato não suportado. Use arquivos .tcx ou .fit".to_string()
            )),
        }
    }

    /// Adiciona o overlay de estatísticas à imagem
    fn add_overlay(&mut self, activity_data: &ActivityData) -> AppResult<()> {
        let font_scale = (self.height as f32 / 40.0).round();
        let scale = Scale::uniform(font_scale);
        let shadow_offset = (font_scale / 15.0).round().max(1.0) as i32;

        // Criamos as linhas de estatísticas usando função estática
        let stats_lines = Self::build_stats_lines_static(activity_data);
        
        // Calculamos o layout usando dados temporários
        let layout = self.calculate_layout(&stats_lines, scale, font_scale);
        
        // Verificamos se é Garmin
        let is_garmin = Self::is_garmin_device_static(&activity_data.device_name);

        // Agora fazemos as operações mutáveis sem conflitos de borrow
        if is_garmin {
            self.add_watermark(&layout)?;
        }

        // Desenha as estatísticas
        self.draw_stats(&stats_lines, &layout, scale, shadow_offset);

        Ok(())
    }

    /// Constrói as linhas de estatísticas com ícones e cores (versão estática)
    fn build_stats_lines_static(activity_data: &ActivityData) -> Vec<(&'static str, String, Rgba<u8>)> {
        let start_time_local = activity_data.start_time.with_timezone(&Local);
        
        vec![
            (ICON_TIME, activity_data.format_duration(), TIME_COLOR),
            (ICON_FIRE, format!("{} kcal", activity_data.calories), CALORIES_COLOR),
            (ICON_HEART, format!("{} avg", activity_data.avg_hr), HR_COLOR),
            (ICON_HEART, format!("{} max", activity_data.max_hr), HR_COLOR),
            (ICON_CALENDAR, start_time_local.format("%d/%m/%Y").to_string(), DATE_COLOR),
            (ICON_DEVICE, activity_data.device_name.clone(), DEVICE_COLOR),
        ]
    }

    /// Calcula o layout e posicionamento do overlay
    fn calculate_layout(&self, stats_lines: &[(&'static str, String, Rgba<u8>)], scale: Scale, font_scale: f32) -> OverlayLayout {
        let padding = (font_scale * 0.75).round() as u32;
        let icon_padding = (padding as f32 * 0.5).round() as i32;
        
        // Calcula a largura máxima das linhas
        let mut max_line_width = 0;
        for (icon, text, _) in stats_lines {
            let (icon_width, _) = text_size(scale, &self.icon_font, icon);
            let (text_width, _) = text_size(scale, &self.font, text);
            let total_line_width = icon_width + icon_padding + text_width;
            if total_line_width > max_line_width {
                max_line_width = total_line_width;
            }
        }

        let text_line_height = font_scale as u32 + (padding / 2);
        let total_text_height = stats_lines.len() as u32 * text_line_height;
        let margin = 4;

        OverlayLayout {
            max_line_width,
            icon_padding,
            text_line_height,
            total_text_height,
            block_x_start: (self.width as i32) - max_line_width - (margin as i32),
            block_y_start: (self.height - total_text_height - margin) as i32,
        }
    }

    /// Verifica se o dispositivo é da marca Garmin (versão estática)
    fn is_garmin_device_static(device_name: &str) -> bool {
        let device_name_lower = device_name.to_lowercase();
        GARMIN_SERIES.iter().any(|&series| device_name_lower.contains(series))
    }

    /// Adiciona marca d'água para dispositivos Garmin
    fn add_watermark(&mut self, layout: &OverlayLayout) -> AppResult<()> {
        println!("🎯 Dispositivo Garmin detectado. Processando marca d'água...");
        println!("📁 Diretório atual: {:?}", std::env::current_dir().unwrap_or_default());
        
        // Primeiro, vamos listar o que existe no diretório img
        if let Ok(entries) = std::fs::read_dir("img") {
            println!("📂 Conteúdo do diretório img/:");
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("   - {:?}", entry.file_name());
                }
            }
        } else {
            println!("❌ Diretório img/ não existe ou não pode ser lido");
        }

        let watermark_width = layout.max_line_width as u32;
        println!("📏 Largura calculada para marca d'água: {}px", watermark_width);
        
        let watermark_height = Self::calculate_watermark_height(watermark_width)?;
        println!("📐 Altura calculada para marca d'água: {}px", watermark_height);
        
        // Se não conseguiu calcular altura, significa que as imagens não existem
        if watermark_height == 0 {
            println!("⚠️  ERRO: Altura da marca d'água é 0 - imagens não encontradas");
            return Ok(()); // Continua sem a marca d'água
        }
        
        let watermark_x = layout.block_x_start as u32;
        let watermark_y = layout.block_y_start as u32 + layout.total_text_height;
        println!("📍 Posição da marca d'água: x={}, y={}", watermark_x, watermark_y);

        let avg_luminance = self.calculate_background_luminance(
            watermark_x, watermark_y, watermark_width, watermark_height
        );
        println!("💡 Luminância média do fundo: {:.1}", avg_luminance);

        let watermark_filename = if avg_luminance < 128.0 {
            println!("🌙 Fundo escuro detectado. Usando marca d'água branca.");
            WATERMARK_WHITE_PATH
        } else {
            println!("☀️  Fundo claro detectado. Usando marca d'água preta.");
            WATERMARK_BLACK_PATH
        };

        // Usa a nova função para encontrar o arquivo
        if let Some(watermark_path) = Self::find_watermark_file(watermark_filename) {
            println!("🎨 Carregando marca d'água de: {:?}", watermark_path);
            
            match image::open(&watermark_path) {
                Ok(watermark_img) => {
                    let (orig_w, orig_h) = watermark_img.dimensions();
                    println!("🖼️  Dimensões originais da marca d'água: {}x{}", orig_w, orig_h);
                    
                    let resized_watermark = imageops::resize(
                        &watermark_img.to_rgba8(),
                        watermark_width,
                        watermark_height,
                        imageops::FilterType::Lanczos3
                    );
                    println!("🔄 Redimensionada para: {}x{}", watermark_width, watermark_height);
                    
                    imageops::overlay(
                        &mut self.image,
                        &resized_watermark,
                        watermark_x as i64,
                        watermark_y as i64
                    );
                    
                    println!("✅ Marca d'água adicionada com sucesso na posição ({}, {})!", watermark_x, watermark_y);
                }
                Err(e) => {
                    println!("❌ Erro ao carregar marca d'água '{:?}': {}", watermark_path, e);
                    println!("   Continuando sem marca d'água...");
                }
            }
        } else {
            println!("🚫 Nenhum arquivo de marca d'água encontrado para '{}'", watermark_filename);
        }

        Ok(())
    }

    /// Encontra o caminho correto para um arquivo de marca d'água
    fn find_watermark_file(filename: &str) -> Option<PathBuf> {
        println!("🔍 Procurando por: {}", filename);
        
        // Lista de diretórios para procurar (em ordem de prioridade)
        let mut search_paths = vec![
            // 1. Caminho exato como especificado
            PathBuf::from(filename),
            // 2. Pasta img no diretório atual  
            PathBuf::from("img").join(std::path::Path::new(filename).file_name().unwrap_or_default()),
        ];

        // 3. Adiciona caminhos baseados no diretório atual (se disponível)
        if let Ok(current_dir) = std::env::current_dir() {
            search_paths.push(current_dir.join(filename));
            search_paths.push(current_dir.join("img").join(std::path::Path::new(filename).file_name().unwrap_or_default()));
        }

        for (i, path) in search_paths.iter().enumerate() {
            println!("   {}. Tentando: {:?}", i + 1, path);
            if path.exists() {
                println!("   ✅ ENCONTRADO em: {:?}", path);
                return Some(path.clone());
            } else {
                println!("   ❌ Não existe");
            }
        }

        println!("   🚫 Arquivo não encontrado em nenhum local");
        None
    }

    /// Calcula a altura da marca d'água mantendo proporção
    fn calculate_watermark_height(width: u32) -> AppResult<u32> {
        println!("🔍 Calculando altura da marca d'água para largura: {}px", width);
        
        let files = [WATERMARK_WHITE_PATH, WATERMARK_BLACK_PATH];
        
        for filename in &files {
            println!("   Tentando arquivo: {}", filename);
            
            if let Some(path) = Self::find_watermark_file(filename) {
                println!("   Arquivo encontrado: {:?}", path);
                
                match image::open(&path) {
                    Ok(img) => {
                        let (orig_w, orig_h) = img.dimensions();
                        let calculated_height = if orig_w > 0 { width * orig_h / orig_w } else { 0 };
                        
                        println!("   Dimensões originais: {}x{}", orig_w, orig_h);
                        println!("   Altura calculada: {}", calculated_height);
                        
                        return Ok(calculated_height);
                    }
                    Err(e) => {
                        println!("   ❌ Erro ao abrir imagem: {}", e);
                    }
                }
            } else {
                println!("   ❌ Arquivo não encontrado");
            }
        }
        
        println!("⚠️  Nenhuma marca d'água válida encontrada - retornando altura 0");
        Ok(0)
    }

    /// Calcula a luminância média do fundo para escolher a marca d'água adequada
    fn calculate_background_luminance(&self, x: u32, y: u32, width: u32, height: u32) -> f32 {
        let mut total_luminance = 0.0;
        let mut pixel_count = 0;

        for px in x..(x + width) {
            for py in y..(y + height) {
                if px < self.width && py < self.height {
                    let pixel = self.image.get_pixel(px, py);
                    let luminance = 0.2126 * (pixel[0] as f32) 
                                  + 0.7152 * (pixel[1] as f32) 
                                  + 0.0722 * (pixel[2] as f32);
                    total_luminance += luminance;
                    pixel_count += 1;
                }
            }
        }

        if pixel_count > 0 { total_luminance / pixel_count as f32 } else { 0.0 }
    }

    /// Desenha as estatísticas na imagem
    fn draw_stats(&mut self, stats_lines: &[(&'static str, String, Rgba<u8>)], layout: &OverlayLayout, scale: Scale, shadow_offset: i32) {
        let mut y_pos = layout.block_y_start;

        for (icon, text, icon_color) in stats_lines {
            let (icon_width, _) = text_size(scale, &self.icon_font, icon);
            let (text_width, _) = text_size(scale, &self.font, text);
            
            let current_line_width = icon_width + layout.icon_padding + text_width;
            let line_x_start = layout.block_x_start + (layout.max_line_width - current_line_width);

            let icon_x = line_x_start;
            let text_x = line_x_start + icon_width + layout.icon_padding;

            // Desenha sombra para melhor legibilidade
            draw_text_mut(&mut self.image, SHADOW_COLOR, icon_x + shadow_offset, y_pos + shadow_offset, scale, &self.icon_font, icon);
            draw_text_mut(&mut self.image, SHADOW_COLOR, text_x + shadow_offset, y_pos + shadow_offset, scale, &self.font, text);

            // Desenha texto principal
            draw_text_mut(&mut self.image, *icon_color, icon_x, y_pos, scale, &self.icon_font, icon);
            draw_text_mut(&mut self.image, TEXT_COLOR, text_x, y_pos, scale, &self.font, text);
            
            y_pos += layout.text_line_height as i32;
        }
    }

    /// Salva a imagem processada
    pub fn save_result(&self) -> AppResult<()> {
        println!("Salvando imagem final em: {:?}", self.output_path);
        
        // Verifica se o diretório pai existe
        if let Some(parent) = self.output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        
        // Verifica se o caminho tem uma extensão válida
        if self.output_path.extension().is_none() {
            return Err(AppError::InvalidFormat(
                format!("Caminho de saída deve incluir uma extensão de arquivo (ex: .png, .jpg): {:?}", 
                       self.output_path)
            ));
        }
        
        self.image.save(&self.output_path)
            .map_err(|e| AppError::ImageError(e))?;
        
        println!("✅ Imagem salva com sucesso!");
        Ok(())
    }
}