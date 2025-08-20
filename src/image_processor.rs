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
        
        println!("📐 [DEBUG] Dimensões da imagem carregada: {}x{}", width, height);

        println!("Carregando fontes...");
        let font = Self::load_font(FONT_PATH)?;
        let icon_font = Self::load_font(ICON_FONT_PATH)?;

        // Gera automaticamente o caminho de saída baseado na imagem original
        let auto_output_path = Self::generate_output_path(image_path)?;

        Ok(Self {
            image,
            width,
            height,
            font,
            icon_font,
            output_path: auto_output_path,
        })
    }

    /// Gera automaticamente o caminho de saída profissional
    fn generate_output_path(image_path: &PathBuf) -> AppResult<PathBuf> {
        use chrono::Local;
        
        // Obter diretório home do usuário
        let home_dir = dirs::home_dir()
            .ok_or_else(|| AppError::InvalidFormat("Não foi possível determinar o diretório home do usuário".to_string()))?;
        
        // Criar estrutura: ~/stats_overlay/YYYY-MM-DD/
        let today = Local::now().format("%Y-%m-%d").to_string();
        let output_dir = home_dir.join("stats_overlay").join(&today);
        
        // Criar diretórios se não existirem
        std::fs::create_dir_all(&output_dir)?;
        
        // Gerar nome do arquivo: nome-original-stats-overlay.ext
        let original_filename = image_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| AppError::InvalidFormat("Nome de arquivo inválido".to_string()))?;
            
        let original_extension = image_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("jpg"); // Default para jpg se não tiver extensão
            
        let new_filename = format!("{}-stats-overlay.{}", original_filename, original_extension);
        let output_path = output_dir.join(&new_filename);
        
        println!("📁 Diretório de saída: {:?}", output_dir);
        println!("📄 Arquivo de saída: {}", new_filename);
        
        Ok(output_path)
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
        println!("📐 [DEBUG] Iniciando overlay - Dimensões atuais da imagem: {}x{}", self.width, self.height);
        
        let font_scale = (self.height as f32 / 40.0).round();
        let scale = Scale::uniform(font_scale);
        let shadow_offset = (font_scale / 15.0).round().max(1.0) as i32;

        println!("📐 [DEBUG] Font scale calculado: {}", font_scale);
        println!("📐 [DEBUG] Shadow offset: {}", shadow_offset);

        // Criamos as linhas de estatísticas usando função estática
        let stats_lines = Self::build_stats_lines_static(activity_data);
        
        // Calculamos o layout usando dados temporários
        let layout = self.calculate_layout(&stats_lines, scale, font_scale);
        
        println!("📐 [DEBUG] Layout calculado - max_width: {}, total_height: {}", 
                 layout.max_line_width, layout.total_text_height);
        
        // Verificamos se é Garmin
        let is_garmin = Self::is_garmin_device_static(&activity_data.device_name);

        // Agora fazemos as operações mutáveis sem conflitos de borrow
        if is_garmin {
            self.add_watermark(&layout)?;
        }

        // Desenha as estatísticas
        self.draw_stats(&stats_lines, &layout, scale, shadow_offset);

        println!("📐 [DEBUG] Overlay concluído - Dimensões finais da imagem: {}x{}", self.width, self.height);

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
        let stats_bottom_margin = 80; // Estatísticas ficam a 80px da borda inferior (50px + 30px)

        OverlayLayout {
            max_line_width,
            icon_padding,
            text_line_height,
            total_text_height,
            block_x_start: (self.width as i32) - max_line_width - (margin as i32),
            block_y_start: (self.height - total_text_height - stats_bottom_margin) as i32,
        }
    }

    /// Verifica se o dispositivo é da marca Garmin (versão estática)
    fn is_garmin_device_static(device_name: &str) -> bool {
        println!("🔍 [GARMIN DEBUG] Verificando dispositivo: '{}'", device_name);
        let device_name_lower = device_name.to_lowercase();
        println!("🔍 [GARMIN DEBUG] Nome em minúsculas: '{}'", device_name_lower);
        
        for series in GARMIN_SERIES {
            if device_name_lower.contains(series) {
                println!("✅ [GARMIN DEBUG] Dispositivo Garmin detectado! Contém: '{}'", series);
                return true;
            }
        }
        
        println!("❌ [GARMIN DEBUG] Dispositivo NÃO é Garmin");
        println!("🔍 [GARMIN DEBUG] Séries verificadas: {:?}", GARMIN_SERIES);
        false
    }

    /// Adiciona marca d'água para dispositivos Garmin (lógica original restaurada)
    fn add_watermark(&mut self, layout: &OverlayLayout) -> AppResult<()> {
        println!("🎯 [DEBUG] Iniciando processo de marca d'água");
        println!("🎯 [DEBUG] Layout recebido: max_width={}, total_height={}", layout.max_line_width, layout.total_text_height);
        
        println!("Dispositivo Garmin detectado. Analisando fundo para a marca d'água.");

        let temp_watermark_width = layout.max_line_width as u32;
        println!("🎯 [DEBUG] Largura da marca d'água: {}", temp_watermark_width);

        println!("🎯 [DEBUG] Tentando abrir: {}", WATERMARK_WHITE_PATH);
        let (wm_orig_w, wm_orig_h) = match image::open(WATERMARK_WHITE_PATH) {
            Ok(img) => {
                let dims = img.dimensions();
                println!("✅ [DEBUG] {} aberto com sucesso: {}x{}", WATERMARK_WHITE_PATH, dims.0, dims.1);
                dims
            },
            Err(e) => {
                println!("❌ [DEBUG] Erro ao abrir {}: {}", WATERMARK_WHITE_PATH, e);
                println!("🎯 [DEBUG] Tentando abrir: {}", WATERMARK_BLACK_PATH);
                match image::open(WATERMARK_BLACK_PATH) {
                    Ok(img) => {
                        let dims = img.dimensions();
                        println!("✅ [DEBUG] {} aberto com sucesso: {}x{}", WATERMARK_BLACK_PATH, dims.0, dims.1);
                        dims
                    },
                    Err(e2) => {
                        println!("❌ [DEBUG] Erro ao abrir {}: {}", WATERMARK_BLACK_PATH, e2);
                        println!("🚫 [DEBUG] NENHUMA marca d'água encontrada - retornando");
                        (1, 1)
                    }
                }
            }
        };

        println!("🎯 [DEBUG] Dimensões da marca d'água: {}x{}", wm_orig_w, wm_orig_h);

        let watermark_height = if wm_orig_w > 0 { temp_watermark_width * wm_orig_h / wm_orig_w } else { 0 };
        println!("🎯 [DEBUG] Altura calculada: {}", watermark_height);

        if watermark_height == 0 {
            println!("🚫 [DEBUG] Altura é 0 - saindo sem marca d'água");
            return Ok(());
        }

        // AJUSTE: Posicionamento dentro da área visível da imagem
        let margin_right = 5i32; // -10px (move para dentro da borda direita)
        let watermark_bottom_margin = 10; // -10px (move para dentro da borda inferior)
        
        // As estatísticas já estão posicionadas corretamente pelo layout
        let block_x_start = (self.width as i32) - (layout.max_line_width) - 20i32; // Mantém margin original para stats
        
        println!("🎯 [DEBUG] Posições calculadas: block_x={}", block_x_start);
        
        let watermark_x = (self.width as i32 - temp_watermark_width as i32 + margin_right) as u32;
        // AJUSTE: Marca d'água fica próxima da borda inferior (valor negativo = mais próximo)
        let watermark_y = (self.height as i32 - watermark_height as i32 + watermark_bottom_margin) as u32;

        println!("🎯 [DEBUG] Posição final da marca d'água: x={}, y={}", watermark_x, watermark_y);
        println!("🎯 [DEBUG] Dimensões da imagem: {}x{}", self.width, self.height);
        println!("🎯 [DEBUG] Área da marca d'água: {}x{} na posição ({}, {})", temp_watermark_width, watermark_height, watermark_x, watermark_y);

        // Verificação de bounds
        if watermark_x >= self.width || watermark_y >= self.height {
            println!("🚫 [DEBUG] Marca d'água fora dos limites da imagem!");
            return Ok(());
        }

        let mut total_luminance = 0.0;
        let mut pixel_count = 0;

        println!("🎯 [DEBUG] Analisando luminância da região: x={} a {}, y={} a {}", 
                 watermark_x, watermark_x + temp_watermark_width, 
                 watermark_y, watermark_y + watermark_height);

        for x in watermark_x..(watermark_x + temp_watermark_width) {
            for y in watermark_y..(watermark_y + watermark_height) {
                if x < self.width && y < self.height {
                    let pixel = self.image.get_pixel(x, y);
                    let luminance = 0.2126 * (pixel[0] as f32) + 0.7152 * (pixel[1] as f32) + 0.0722 * (pixel[2] as f32);
                    total_luminance += luminance;
                    pixel_count += 1;
                }
            }
        }

        let avg_luminance = if pixel_count > 0 { total_luminance / pixel_count as f32 } else { 0.0 };
        println!("🎯 [DEBUG] Luminância média: {:.1} (pixels analisados: {})", avg_luminance, pixel_count);
        
        let watermark_path_to_use = if avg_luminance < 128.0 {
            println!("Fundo escuro detectado. Usando marca d'água branca.");
            WATERMARK_WHITE_PATH
        } else {
            println!("Fundo claro detectado. Usando marca d'água preta.");
            WATERMARK_BLACK_PATH
        };

        println!("🎯 [DEBUG] Tentando carregar marca d'água final: {}", watermark_path_to_use);

        if let Ok(watermark_img_orig) = image::open(watermark_path_to_use) {
            println!("✅ [DEBUG] Marca d'água carregada com sucesso!");
            let watermark_img = watermark_img_orig.to_rgba8();
            let resized_watermark = imageops::resize(
                &watermark_img,
                temp_watermark_width,
                watermark_height,
                imageops::FilterType::Lanczos3
            );
            
            println!("🎯 [DEBUG] Marca d'água redimensionada para: {}x{}", temp_watermark_width, watermark_height);
            println!("🎯 [DEBUG] Aplicando overlay na posição: ({}, {})", watermark_x, watermark_y);
            
            imageops::overlay(
                &mut self.image,
                &resized_watermark,
                watermark_x as i64,
                watermark_y as i64
            );
            
            println!("✅ Marca d'água adicionada com sucesso!");
        } else {
            println!("❌ [DEBUG] FALHA ao abrir marca d'água final: {}", watermark_path_to_use);
            println!("Aviso: Imagem da marca d'água não encontrada em '{}'.", watermark_path_to_use);
        }

        Ok(())
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
        println!("📐 [DEBUG] Salvando imagem - Dimensões antes do salvamento: {}x{}", self.width, self.height);
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
        
        // Log das dimensões da imagem antes de salvar
        let (final_width, final_height) = self.image.dimensions();
        println!("📐 [DEBUG] Dimensões da imagem no buffer: {}x{}", final_width, final_height);
        
        self.image.save(&self.output_path)
            .map_err(|e| AppError::ImageError(e))?;
        
        // Verificar o arquivo salvo
        if let Ok(saved_img) = image::open(&self.output_path) {
            let (saved_w, saved_h) = saved_img.dimensions();
            println!("📐 [DEBUG] Dimensões da imagem salva: {}x{}", saved_w, saved_h);
        }
        
        println!("✅ Imagem salva com sucesso!");
        Ok(())
    }
}