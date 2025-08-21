// ============================================================================
// src/image_processor.rs - Processamento de imagens e overlay (VERSÃO CORRIGIDA)
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

/// Enum para diferentes tipos de linha de estatística
#[derive(Debug, Clone)]
enum StatLine {
    Simple {
        icon: &'static str,
        text: String,
        color: Rgba<u8>,
    },
    WithSubtext {
        icon: &'static str,
        main_text: String,
        sub_text: String,
        main_color: Rgba<u8>,
        sub_color: Rgba<u8>,
    },
}

/// Processador principal de imagens com overlay de estatísticas
pub struct ImageProcessor {
    image: RgbaImage,
    width: u32,
    height: u32,
    font: Font<'static>,
    icon_font: Font<'static>,
    output_path: PathBuf,
}

/// Layout para posicionamento do overlay com posições absolutas fixas
#[derive(Debug)]
struct OverlayLayout {
    // Estatísticas
    stats_x: u32,
    stats_y: u32,
    stats_width: u32,
    stats_height: u32,
    
    // Marca d'água
    watermark_x: u32,
    watermark_y: u32,
    watermark_width: u32,
    watermark_height: u32,
    
    // Layout interno das estatísticas
    max_line_width: i32,
    icon_padding: i32,
    text_line_height: u32,
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
        
        // Calcula o tamanho da fonte baseado na menor dimensão da imagem
        let font_scale = (self.height.min(self.width) as f32 / 40.0).round().max(12.0);
        let scale = Scale::uniform(font_scale);
        let shadow_offset = (font_scale / 15.0).round().max(1.0) as i32;

        println!("📐 [DEBUG] Font scale calculado: {}", font_scale);
        println!("📐 [DEBUG] Shadow offset: {}", shadow_offset);

        // Criamos as linhas de estatísticas
        let stats_lines = Self::build_stats_lines_static(activity_data);
        
        // Calculamos o layout com posicionamento fixo
        let layout = self.calculate_fixed_layout(&stats_lines, scale, font_scale)?;
        
        println!("📐 [DEBUG] Layout calculado:");
        println!("   Stats: {}x{} na posição ({}, {})", layout.stats_width, layout.stats_height, layout.stats_x, layout.stats_y);
        println!("   Watermark: {}x{} na posição ({}, {})", layout.watermark_width, layout.watermark_height, layout.watermark_x, layout.watermark_y);
        
        // Verificamos se é Garmin e adiciona marca d'água primeiro
        let is_garmin = Self::is_garmin_device_static(&activity_data.device_name);
        if is_garmin {
            self.add_watermark_fixed(&layout)?;
        }

        // Desenha as estatísticas
        self.draw_stats_fixed(&stats_lines, &layout, scale, shadow_offset);

        println!("📐 [DEBUG] Overlay concluído - Dimensões finais da imagem: {}x{}", self.width, self.height);

        Ok(())
    }

    /// Constrói as linhas de estatísticas com ícones e cores (versão estática)
    fn build_stats_lines_static(activity_data: &ActivityData) -> Vec<StatLine> {
        let start_time_local = activity_data.start_time.with_timezone(&Local);
        
        vec![
            StatLine::Simple {
                icon: ICON_TIME,
                text: activity_data.format_duration(),
                color: TIME_COLOR,
            },
            StatLine::Simple {
                icon: ICON_FIRE,
                text: format!("{} kcal", activity_data.calories),
                color: CALORIES_COLOR,
            },
            StatLine::Simple {
                icon: ICON_HEART,
                text: format!("{} avg", activity_data.avg_hr),
                color: HR_COLOR,
            },
            StatLine::Simple {
                icon: ICON_HEART,
                text: format!("{} max", activity_data.max_hr),
                color: HR_COLOR,
            },
            StatLine::WithSubtext {
                icon: ICON_CALENDAR,
                main_text: start_time_local.format("%H:%M").to_string(),
                sub_text: start_time_local.format("%d/%m/%Y").to_string(),
                main_color: DATE_COLOR,
                sub_color: Rgba([180u8, 180u8, 180u8, 255u8]), // Cor mais suave para a data
            },
            StatLine::Simple {
                icon: ICON_DEVICE,
                text: activity_data.device_name.clone(),
                color: DEVICE_COLOR,
            },
        ]
    }

    /// Calcula o layout com posicionamento absolutamente fixo
    fn calculate_fixed_layout(&self, stats_lines: &[StatLine], scale: Scale, font_scale: f32) -> AppResult<OverlayLayout> {
        // Margens fixas a partir das bordas da imagem
        const BOTTOM_MARGIN: u32 = 20;  // Distância da borda inferior
        const RIGHT_MARGIN: u32 = 20;   // Distância da borda direita
        const STATS_WATERMARK_GAP: u32 = 20; // Gap entre estatísticas e marca d'água

        let padding = (font_scale * 0.75).round() as u32;
        let icon_padding = (padding as f32 * 0.5).round() as i32;
        
        // Calcula a largura máxima das linhas de estatísticas
        let mut max_line_width = 0;
        let mut total_height = 0;
        
        for stat_line in stats_lines {
            match stat_line {
                StatLine::Simple { icon, text, .. } => {
                    let (icon_width, _) = text_size(scale, &self.icon_font, icon);
                    let (text_width, _) = text_size(scale, &self.font, text);
                    let total_line_width = icon_width + icon_padding + text_width;
                    if total_line_width > max_line_width {
                        max_line_width = total_line_width;
                    }
                    total_height += font_scale as u32 + (padding / 2);
                },
                StatLine::WithSubtext { icon, main_text, sub_text, .. } => {
                    let (icon_width, _) = text_size(scale, &self.icon_font, icon);
                    let (main_text_width, _) = text_size(scale, &self.font, main_text);
                    
                    // Calcula a largura do subtexto com fonte menor
                    let sub_scale = Scale::uniform(font_scale * 0.75);
                    let (sub_text_width, _) = text_size(sub_scale, &self.font, sub_text);
                    
                    // A largura total é a maior entre texto principal e subtexto
                    let max_text_width = main_text_width.max(sub_text_width);
                    let total_line_width = icon_width + icon_padding + max_text_width;
                    if total_line_width > max_line_width {
                        max_line_width = total_line_width;
                    }
                    
                    // Altura para linha principal + sublinha (com espaçamento menor)
                    total_height += font_scale as u32 + (font_scale * 0.75) as u32 + (padding / 2);
                }
            }
        }

        let stats_height = total_height;
        let stats_width = max_line_width as u32;

        // Calcula as dimensões da marca d'água (tentativa com imagem padrão)
        let (watermark_width, watermark_height) = self.calculate_watermark_dimensions(stats_width)?;

        // POSICIONAMENTO FIXO:
        // 1. Marca d'água: sempre no canto inferior direito
        let watermark_x = self.width.saturating_sub(watermark_width + RIGHT_MARGIN);
        let watermark_y = self.height.saturating_sub(watermark_height + BOTTOM_MARGIN);

        // 2. Estatísticas: sempre acima da marca d'água com gap fixo
        let stats_x = self.width.saturating_sub(stats_width + RIGHT_MARGIN);
        let stats_y = watermark_y.saturating_sub(stats_height + STATS_WATERMARK_GAP);

        println!("📐 [LAYOUT DEBUG] Cálculos de posicionamento:");
        println!("   Imagem: {}x{}", self.width, self.height);
        println!("   Stats calculadas: {}x{}", stats_width, stats_height);
        println!("   Watermark calculada: {}x{}", watermark_width, watermark_height);
        println!("   Margens: bottom={}, right={}, gap={}", BOTTOM_MARGIN, RIGHT_MARGIN, STATS_WATERMARK_GAP);

        Ok(OverlayLayout {
            stats_x,
            stats_y,
            stats_width,
            stats_height,
            watermark_x,
            watermark_y,
            watermark_width,
            watermark_height,
            max_line_width,
            icon_padding,
            text_line_height: font_scale as u32 + (padding / 2), // Mantém para compatibilidade
        })
    }

    /// Calcula as dimensões da marca d'água baseado no tamanho das estatísticas
    fn calculate_watermark_dimensions(&self, stats_width: u32) -> AppResult<(u32, u32)> {
        // Tenta abrir uma das imagens de marca d'água para obter as proporções originais
        let watermark_path = if std::path::Path::new(WATERMARK_WHITE_PATH).exists() {
            WATERMARK_WHITE_PATH
        } else if std::path::Path::new(WATERMARK_BLACK_PATH).exists() {
            WATERMARK_BLACK_PATH
        } else {
            // Se não encontrar nenhuma marca d'água, usa dimensões padrão
            println!("⚠️  [DEBUG] Nenhuma marca d'água encontrada, usando dimensões padrão");
            return Ok((stats_width, stats_width / 4)); // Proporção 4:1
        };

        match image::open(watermark_path) {
            Ok(img) => {
                let (orig_w, orig_h) = img.dimensions();
                println!("📐 [DEBUG] Marca d'água original: {}x{}", orig_w, orig_h);
                
                // A marca d'água terá a mesma largura que as estatísticas
                let watermark_width = stats_width;
                let watermark_height = if orig_w > 0 { 
                    watermark_width * orig_h / orig_w 
                } else { 
                    watermark_width / 4 
                };
                
                println!("📐 [DEBUG] Marca d'água redimensionada: {}x{}", watermark_width, watermark_height);
                Ok((watermark_width, watermark_height))
            },
            Err(e) => {
                println!("⚠️  [DEBUG] Erro ao abrir marca d'água {}: {}", watermark_path, e);
                // Usa proporção padrão se não conseguir abrir
                Ok((stats_width, stats_width / 4))
            }
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

    /// Adiciona marca d'água com posicionamento fixo
    fn add_watermark_fixed(&mut self, layout: &OverlayLayout) -> AppResult<()> {
        println!("🎯 [DEBUG] Iniciando processo de marca d'água com posicionamento fixo");
        println!("🎯 [DEBUG] Posição da marca d'água: ({}, {})", layout.watermark_x, layout.watermark_y);
        println!("🎯 [DEBUG] Dimensões da marca d'água: {}x{}", layout.watermark_width, layout.watermark_height);

        println!("Dispositivo Garmin detectado. Analisando fundo para a marca d'água.");

        // Análise da luminância da região onde a marca d'água será colocada
        let mut total_luminance = 0.0;
        let mut pixel_count = 0;

        let end_x = (layout.watermark_x + layout.watermark_width).min(self.width);
        let end_y = (layout.watermark_y + layout.watermark_height).min(self.height);

        println!("🎯 [DEBUG] Analisando luminância da região: x={} a {}, y={} a {}", 
                 layout.watermark_x, end_x, layout.watermark_y, end_y);

        for x in layout.watermark_x..end_x {
            for y in layout.watermark_y..end_y {
                let pixel = self.image.get_pixel(x, y);
                let luminance = 0.2126 * (pixel[0] as f32) + 0.7152 * (pixel[1] as f32) + 0.0722 * (pixel[2] as f32);
                total_luminance += luminance;
                pixel_count += 1;
            }
        }

        let avg_luminance = if pixel_count > 0 { total_luminance / pixel_count as f32 } else { 128.0 };
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
                layout.watermark_width,
                layout.watermark_height,
                imageops::FilterType::Lanczos3
            );
            
            println!("🎯 [DEBUG] Marca d'água redimensionada para: {}x{}", layout.watermark_width, layout.watermark_height);
            println!("🎯 [DEBUG] Aplicando overlay na posição: ({}, {})", layout.watermark_x, layout.watermark_y);
            
            // Verifica bounds antes de aplicar
            if layout.watermark_x < self.width && layout.watermark_y < self.height {
                imageops::overlay(
                    &mut self.image,
                    &resized_watermark,
                    layout.watermark_x as i64,
                    layout.watermark_y as i64
                );
                
                println!("✅ Marca d'água adicionada com sucesso!");
            } else {
                println!("🚫 [DEBUG] Marca d'água fora dos limites da imagem!");
            }
        } else {
            println!("❌ [DEBUG] FALHA ao abrir marca d'água final: {}", watermark_path_to_use);
            println!("Aviso: Imagem da marca d'água não encontrada em '{}'.", watermark_path_to_use);
        }

        Ok(())
    }

    /// Desenha as estatísticas na imagem com posicionamento fixo
    fn draw_stats_fixed(&mut self, stats_lines: &[StatLine], layout: &OverlayLayout, scale: Scale, shadow_offset: i32) {
        println!("📝 [DEBUG] Desenhando estatísticas na posição: ({}, {})", layout.stats_x, layout.stats_y);
        
        let mut y_pos = layout.stats_y as i32;
        let font_scale = scale.x; // Obtém o valor da escala

        for stat_line in stats_lines {
            match stat_line {
                StatLine::Simple { icon, text, color } => {
                    let (icon_width, _) = text_size(scale, &self.icon_font, icon);
                    let (text_width, _) = text_size(scale, &self.font, text);
                    
                    let current_line_width = icon_width + layout.icon_padding + text_width;
                    
                    // Alinhamento à direita: calcula posição x baseada na largura total das estatísticas
                    let line_x_start = (layout.stats_x as i32) + (layout.max_line_width - current_line_width);

                    let icon_x = line_x_start;
                    let text_x = line_x_start + icon_width + layout.icon_padding;

                    // Verifica bounds antes de desenhar
                    if icon_x >= 0 && text_x >= 0 && y_pos >= 0 {
                        // Desenha sombra para melhor legibilidade
                        draw_text_mut(&mut self.image, SHADOW_COLOR, icon_x + shadow_offset, y_pos + shadow_offset, scale, &self.icon_font, icon);
                        draw_text_mut(&mut self.image, SHADOW_COLOR, text_x + shadow_offset, y_pos + shadow_offset, scale, &self.font, text);

                        // Desenha texto principal
                        draw_text_mut(&mut self.image, *color, icon_x, y_pos, scale, &self.icon_font, icon);
                        draw_text_mut(&mut self.image, TEXT_COLOR, text_x, y_pos, scale, &self.font, text);
                    }
                    
                    y_pos += layout.text_line_height as i32;
                },
                StatLine::WithSubtext { icon, main_text, sub_text, main_color, sub_color } => {
                    let (icon_width, _) = text_size(scale, &self.icon_font, icon);
                    let (main_text_width, _) = text_size(scale, &self.font, main_text);
                    
                    // Escala menor para o subtexto
                    let sub_scale = Scale::uniform(font_scale * 0.75);
                    let (sub_text_width, _) = text_size(sub_scale, &self.font, sub_text);
                    
                    // Calcula a largura total considerando a maior largura entre textos
                    let max_text_width = main_text_width.max(sub_text_width);
                    let current_line_width = icon_width + layout.icon_padding + max_text_width;
                    
                    // Alinhamento à direita
                    let line_x_start = (layout.stats_x as i32) + (layout.max_line_width - current_line_width);
                    let icon_x = line_x_start;
                    let main_text_x = line_x_start + icon_width + layout.icon_padding;
                    
                    // Posição do subtexto (alinhado com o texto principal)
                    let sub_text_x = main_text_x;
                    let sub_text_y = y_pos + (font_scale * 1.0) as i32; // Posicionado abaixo do texto principal

                    // Verifica bounds antes de desenhar
                    if icon_x >= 0 && main_text_x >= 0 && y_pos >= 0 {
                        // === DESENHA ÍCONE ===
                        // Sombra do ícone
                        draw_text_mut(&mut self.image, SHADOW_COLOR, icon_x + shadow_offset, y_pos + shadow_offset, scale, &self.icon_font, icon);
                        // Ícone principal
                        draw_text_mut(&mut self.image, *main_color, icon_x, y_pos, scale, &self.icon_font, icon);
                        
                        // === DESENHA TEXTO PRINCIPAL (HORÁRIO) ===
                        // Sombra do texto principal
                        draw_text_mut(&mut self.image, SHADOW_COLOR, main_text_x + shadow_offset, y_pos + shadow_offset, scale, &self.font, main_text);
                        // Texto principal
                        draw_text_mut(&mut self.image, TEXT_COLOR, main_text_x, y_pos, scale, &self.font, main_text);
                        
                        // === DESENHA SUBTEXTO (DATA) ===
                        if sub_text_y >= 0 {
                            // Sombra do subtexto
                            draw_text_mut(&mut self.image, SHADOW_COLOR, sub_text_x + shadow_offset, sub_text_y + shadow_offset, sub_scale, &self.font, sub_text);
                            // Subtexto
                            draw_text_mut(&mut self.image, *sub_color, sub_text_x, sub_text_y, sub_scale, &self.font, sub_text);
                        }
                    }
                    
                    // Incrementa Y para a próxima linha (considerando altura do texto principal + subtexto)
                    y_pos += (font_scale * 1.75) as i32 + (layout.text_line_height as i32 / 4);
                }
            }
        }
        
        println!("✅ [DEBUG] Estatísticas desenhadas com sucesso!");
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