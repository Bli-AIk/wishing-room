use ply_engine::prelude::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub(crate) enum ThemeChoice {
    System,
    Dark,
    Light,
    CatppuccinLatte,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ThemeAppearance {
    Light,
    Dark,
}

/// Theme palette using Ply Color values instead of CSS strings.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct PlyTheme {
    pub(crate) name: String,
    pub(crate) appearance: ThemeAppearance,
    pub(crate) background: Color,
    pub(crate) background_elevated: Color,
    pub(crate) surface: Color,
    pub(crate) surface_elevated: Color,
    pub(crate) surface_overlay: Color,
    pub(crate) border: Color,
    pub(crate) border_strong: Color,
    pub(crate) text: Color,
    pub(crate) muted_text: Color,
    pub(crate) accent: Color,
    pub(crate) accent_soft: Color,
    pub(crate) accent_text: Color,
    pub(crate) success: Color,
    pub(crate) danger: Color,
    pub(crate) canvas_base: Color,
    pub(crate) empty_tile: Color,
    pub(crate) grid_line: Color,
    pub(crate) selection_overlay: Color,
    pub(crate) shadow: Color,
}

fn hex(rgb: u32) -> Color {
    Color::from(rgb)
}

fn hexa(rgb: u32, alpha: u8) -> Color {
    let r = ((rgb >> 16) & 0xFF) as u8;
    let g = ((rgb >> 8) & 0xFF) as u8;
    let b = (rgb & 0xFF) as u8;
    Color::from((r, g, b, alpha))
}

impl PlyTheme {
    pub(crate) fn from_choice(choice: ThemeChoice, custom: &ThemePaletteData) -> Self {
        match choice {
            ThemeChoice::System => Self::taled_dark(),
            ThemeChoice::Dark => Self::taled_dark(),
            ThemeChoice::Light => Self::taled_light(),
            ThemeChoice::CatppuccinLatte => Self::catppuccin_latte(),
            ThemeChoice::CatppuccinFrappe => Self::catppuccin_frappe(),
            ThemeChoice::CatppuccinMacchiato => Self::catppuccin_macchiato(),
            ThemeChoice::CatppuccinMocha => Self::catppuccin_mocha(),
            ThemeChoice::Custom => Self::from_palette_data(custom),
        }
    }

    pub(crate) fn taled_dark() -> Self {
        Self {
            name: "Taled Dark".to_string(),
            appearance: ThemeAppearance::Dark,
            background: hex(0x121214),
            background_elevated: hex(0x1f1f21),
            surface: hex(0x1c1c1e),
            surface_elevated: hex(0x242426),
            surface_overlay: hexa(0x1c1c1e, 230),
            border: hex(0x2c2c2e),
            border_strong: hex(0x3a3a3c),
            text: hex(0xf2f2f7),
            muted_text: hex(0x8f8f95),
            accent: hex(0x0a84ff),
            accent_soft: hexa(0x0a84ff, 41),
            accent_text: hex(0xffffff),
            success: hex(0x30d158),
            danger: hex(0xff453a),
            canvas_base: hex(0x2a2a2a),
            empty_tile: hex(0x142131),
            grid_line: hexa(0xffffff, 22),
            selection_overlay: hexa(0x000000, 97),
            shadow: hexa(0x000000, 71),
        }
    }

    pub(crate) fn taled_light() -> Self {
        Self {
            name: "Taled Light".to_string(),
            appearance: ThemeAppearance::Light,
            background: hex(0xf5f5f7),
            background_elevated: hex(0xffffff),
            surface: hex(0xffffff),
            surface_elevated: hex(0xf2f2f7),
            surface_overlay: hexa(0xffffff, 235),
            border: hex(0xd7d7dc),
            border_strong: hex(0xc7c7cc),
            text: hex(0x1c1c1e),
            muted_text: hex(0x6e6e73),
            accent: hex(0x0a84ff),
            accent_soft: hexa(0x0a84ff, 31),
            accent_text: hex(0xffffff),
            success: hex(0x248a3d),
            danger: hex(0xc5281c),
            canvas_base: hex(0xececf1),
            empty_tile: hex(0xc8d3e0),
            grid_line: hexa(0x1c1c1e, 23),
            selection_overlay: hexa(0x000000, 36),
            shadow: hexa(0x0f172a, 26),
        }
    }

    pub(crate) fn catppuccin_latte() -> Self {
        Self {
            name: "Catppuccin Latte".to_string(),
            appearance: ThemeAppearance::Light,
            background: hex(0xeff1f5),
            background_elevated: hex(0xe6e9ef),
            surface: hex(0xffffff),
            surface_elevated: hex(0xccd0da),
            surface_overlay: hexa(0xffffff, 235),
            border: hex(0xbcc0cc),
            border_strong: hex(0xacb0be),
            text: hex(0x4c4f69),
            muted_text: hex(0x5c5f77),
            accent: hex(0x1e66f5),
            accent_soft: hexa(0x1e66f5, 36),
            accent_text: hex(0xffffff),
            success: hex(0x40a02b),
            danger: hex(0xd20f39),
            canvas_base: hex(0xdce0e8),
            empty_tile: hex(0xbcc3d4),
            grid_line: hexa(0x4c4f69, 26),
            selection_overlay: hexa(0x4c4f69, 31),
            shadow: hexa(0x4c4f69, 31),
        }
    }

    pub(crate) fn catppuccin_frappe() -> Self {
        Self {
            name: "Catppuccin Frappe".to_string(),
            appearance: ThemeAppearance::Dark,
            background: hex(0x292c3c),
            background_elevated: hex(0x303446),
            surface: hex(0x414559),
            surface_elevated: hex(0x51576d),
            surface_overlay: hexa(0x303446, 235),
            border: hex(0x626880),
            border_strong: hex(0x737994),
            text: hex(0xc6d0f5),
            muted_text: hex(0xa5adce),
            accent: hex(0x8caaee),
            accent_soft: hexa(0x8caaee, 41),
            accent_text: hex(0x232634),
            success: hex(0xa6d189),
            danger: hex(0xe78284),
            canvas_base: hex(0x51576d),
            empty_tile: hex(0x303446),
            grid_line: hexa(0xc6d0f5, 22),
            selection_overlay: hexa(0x232634, 97),
            shadow: hexa(0x000000, 77),
        }
    }

    pub(crate) fn catppuccin_macchiato() -> Self {
        Self {
            name: "Catppuccin Macchiato".to_string(),
            appearance: ThemeAppearance::Dark,
            background: hex(0x1e2030),
            background_elevated: hex(0x24273a),
            surface: hex(0x363a4f),
            surface_elevated: hex(0x494d64),
            surface_overlay: hexa(0x24273a, 235),
            border: hex(0x5b6078),
            border_strong: hex(0x6e738d),
            text: hex(0xcad3f5),
            muted_text: hex(0xa5adcb),
            accent: hex(0x8aadf4),
            accent_soft: hexa(0x8aadf4, 41),
            accent_text: hex(0x181926),
            success: hex(0xa6da95),
            danger: hex(0xed8796),
            canvas_base: hex(0x494d64),
            empty_tile: hex(0x24273a),
            grid_line: hexa(0xcad3f5, 22),
            selection_overlay: hexa(0x181926, 97),
            shadow: hexa(0x000000, 77),
        }
    }

    pub(crate) fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha".to_string(),
            appearance: ThemeAppearance::Dark,
            background: hex(0x181825),
            background_elevated: hex(0x1e1e2e),
            surface: hex(0x313244),
            surface_elevated: hex(0x45475a),
            surface_overlay: hexa(0x1e1e2e, 235),
            border: hex(0x585b70),
            border_strong: hex(0x6c7086),
            text: hex(0xcdd6f4),
            muted_text: hex(0xa6adc8),
            accent: hex(0x89b4fa),
            accent_soft: hexa(0x89b4fa, 41),
            accent_text: hex(0x11111b),
            success: hex(0xa6e3a1),
            danger: hex(0xf38ba8),
            canvas_base: hex(0x45475a),
            empty_tile: hex(0x1e1e2e),
            grid_line: hexa(0xcdd6f4, 22),
            selection_overlay: hexa(0x11111b, 102),
            shadow: hexa(0x000000, 82),
        }
    }

    fn from_palette_data(data: &ThemePaletteData) -> Self {
        Self {
            name: data.name.clone(),
            appearance: data.appearance,
            background: parse_css_color(&data.background),
            background_elevated: parse_css_color(&data.background_elevated),
            surface: parse_css_color(&data.surface),
            surface_elevated: parse_css_color(&data.surface_elevated),
            surface_overlay: parse_css_color(&data.surface_overlay),
            border: parse_css_color(&data.border),
            border_strong: parse_css_color(&data.border_strong),
            text: parse_css_color(&data.text),
            muted_text: parse_css_color(&data.muted_text),
            accent: parse_css_color(&data.accent),
            accent_soft: parse_css_color(&data.accent_soft),
            accent_text: parse_css_color(&data.accent_text),
            success: parse_css_color(&data.success),
            danger: parse_css_color(&data.danger),
            canvas_base: parse_css_color(&data.canvas_base),
            empty_tile: parse_css_color(&data.empty_tile),
            grid_line: parse_css_color(&data.grid_line),
            selection_overlay: parse_css_color(&data.selection_overlay),
            shadow: parse_css_color(&data.shadow),
        }
    }
}

/// Serializable theme data for custom theme import/export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ThemePaletteData {
    pub(crate) name: String,
    pub(crate) appearance: ThemeAppearance,
    pub(crate) background: String,
    pub(crate) background_elevated: String,
    pub(crate) surface: String,
    pub(crate) surface_elevated: String,
    pub(crate) surface_overlay: String,
    pub(crate) border: String,
    pub(crate) border_strong: String,
    pub(crate) text: String,
    pub(crate) muted_text: String,
    pub(crate) accent: String,
    pub(crate) accent_soft: String,
    pub(crate) accent_text: String,
    pub(crate) success: String,
    pub(crate) danger: String,
    pub(crate) canvas_base: String,
    pub(crate) empty_tile: String,
    pub(crate) grid_line: String,
    pub(crate) selection_overlay: String,
    pub(crate) shadow: String,
}

impl ThemePaletteData {
    pub(crate) fn taled_dark() -> Self {
        Self {
            name: "Taled Dark".to_string(),
            appearance: ThemeAppearance::Dark,
            background: "#121214".to_string(),
            background_elevated: "#1f1f21".to_string(),
            surface: "#1c1c1e".to_string(),
            surface_elevated: "#242426".to_string(),
            surface_overlay: "rgba(28, 28, 30, 0.90)".to_string(),
            border: "#2c2c2e".to_string(),
            border_strong: "#3a3a3c".to_string(),
            text: "#f2f2f7".to_string(),
            muted_text: "#8f8f95".to_string(),
            accent: "#0a84ff".to_string(),
            accent_soft: "rgba(10, 132, 255, 0.16)".to_string(),
            accent_text: "#ffffff".to_string(),
            success: "#30d158".to_string(),
            danger: "#ff453a".to_string(),
            canvas_base: "#2a2a2a".to_string(),
            empty_tile: "#142131".to_string(),
            grid_line: "rgba(255,255,255,0.085)".to_string(),
            selection_overlay: "rgba(0,0,0,0.38)".to_string(),
            shadow: "rgba(0,0,0,0.28)".to_string(),
        }
    }
}

pub(crate) fn default_custom_theme() -> ThemePaletteData {
    ThemePaletteData::taled_dark()
}

/// All built-in palettes for preview in the themes browser.
#[allow(dead_code)]
pub(crate) static PALETTES: [fn() -> PlyTheme; 7] = [
    PlyTheme::taled_dark,
    PlyTheme::taled_light,
    PlyTheme::catppuccin_latte,
    PlyTheme::catppuccin_frappe,
    PlyTheme::catppuccin_macchiato,
    PlyTheme::catppuccin_mocha,
    PlyTheme::taled_dark, // placeholder for custom
];

#[allow(dead_code)]
pub(crate) fn export_theme_json(theme: &ThemePaletteData) -> Result<String, String> {
    serde_json::to_string_pretty(theme).map_err(|e| e.to_string())
}

#[allow(dead_code)]
pub(crate) fn import_theme_json(source: &str) -> Result<ThemePaletteData, String> {
    let mut theme = serde_json::from_str::<ThemePaletteData>(source).map_err(|e| e.to_string())?;
    if theme.name.trim().is_empty() {
        theme.name = "Imported Theme".to_string();
    }
    Ok(theme)
}

#[allow(dead_code)]
pub(crate) fn theme_label(choice: ThemeChoice) -> &'static str {
    match choice {
        ThemeChoice::System => "settings-theme-system",
        ThemeChoice::Dark => "settings-theme-dark",
        ThemeChoice::Light => "settings-theme-light",
        ThemeChoice::CatppuccinLatte => "settings-theme-catppuccin-latte",
        ThemeChoice::CatppuccinFrappe => "settings-theme-catppuccin-frappe",
        ThemeChoice::CatppuccinMacchiato => "settings-theme-catppuccin-macchiato",
        ThemeChoice::CatppuccinMocha => "settings-theme-catppuccin-mocha",
        ThemeChoice::Custom => "settings-theme-custom",
    }
}

pub(crate) fn theme_choice_display_label(state: &crate::app_state::AppState) -> String {
    let key = theme_label(state.theme_choice);
    crate::l10n::text(state.resolved_language(), key)
}

fn parse_css_color(s: &str) -> Color {
    let s = s.trim();
    if let Some(hex_str) = s.strip_prefix('#') {
        let val = u32::from_str_radix(hex_str, 16).unwrap_or(0);
        if hex_str.len() == 6 {
            return hex(val);
        }
    }
    if let Some(inner) = s.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 4 {
            let r = parts[0].trim().parse::<u8>().unwrap_or(0);
            let g = parts[1].trim().parse::<u8>().unwrap_or(0);
            let b = parts[2].trim().parse::<u8>().unwrap_or(0);
            let a_f: f32 = parts[3].trim().parse().unwrap_or(1.0);
            let a = (a_f * 255.0) as u8;
            return Color::from((r, g, b, a));
        }
    }
    hex(0x000000)
}
