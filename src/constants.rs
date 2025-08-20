// ============================================================================
// src/constants.rs - Constantes da aplicação
// ============================================================================

use image::Rgba;

// Caminhos de arquivos
pub const FONT_PATH: &str = "fonts/DejaVuSans.ttf";
pub const ICON_FONT_PATH: &str = "fonts/FontAwesome.ttf";
pub const DEFAULT_OUTPUT_PATH: &str = "resultado_com_overlay.png";
pub const WATERMARK_WHITE_PATH: &str = "img/garmin_white.png";
pub const WATERMARK_BLACK_PATH: &str = "img/garmin_black.png";

// Cores para diferentes tipos de dados
pub const TEXT_COLOR: Rgba<u8> = Rgba([255u8, 255u8, 255u8, 255u8]);
pub const SHADOW_COLOR: Rgba<u8> = Rgba([0u8, 0u8, 0u8, 255u8]);
pub const TIME_COLOR: Rgba<u8> = Rgba([52u8, 152u8, 219u8, 255u8]);
pub const CALORIES_COLOR: Rgba<u8> = Rgba([230u8, 126u8, 34u8, 255u8]);
pub const HR_COLOR: Rgba<u8> = Rgba([231u8, 76u8, 60u8, 255u8]);
pub const DATE_COLOR: Rgba<u8> = Rgba([46u8, 204u8, 113u8, 255u8]);
pub const DEVICE_COLOR: Rgba<u8> = Rgba([149u8, 165u8, 166u8, 255u8]);

// Ícones FontAwesome (Unicode)
pub const ICON_TIME: &str = "\u{f017}";
pub const ICON_FIRE: &str = "\u{f06d}";
pub const ICON_HEART: &str = "\u{f21e}";
pub const ICON_CALENDAR: &str = "\u{f133}";
pub const ICON_DEVICE: &str = "\u{f10b}";

// Séries de dispositivos Garmin para detecção
pub const GARMIN_SERIES: &[&str] = &[
    "forerunner", "fenix", "venu", "vivoactive", "instinct",
    "epix", "enduro", "approach", "marq", "lily", "tactix", "descent", "garmin"
];