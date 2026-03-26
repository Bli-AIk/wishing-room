use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ThemePalette {
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
    pub(crate) grid_line: String,
    pub(crate) selection_overlay: String,
    pub(crate) shadow: String,
}

impl ThemePalette {
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
            grid_line: "rgba(255,255,255,0.085)".to_string(),
            selection_overlay: "rgba(0,0,0,0.38)".to_string(),
            shadow: "rgba(0,0,0,0.28)".to_string(),
        }
    }

    pub(crate) fn taled_light() -> Self {
        Self {
            name: "Taled Light".to_string(),
            appearance: ThemeAppearance::Light,
            background: "#f5f5f7".to_string(),
            background_elevated: "#ffffff".to_string(),
            surface: "#ffffff".to_string(),
            surface_elevated: "#f2f2f7".to_string(),
            surface_overlay: "rgba(255, 255, 255, 0.92)".to_string(),
            border: "#d7d7dc".to_string(),
            border_strong: "#c7c7cc".to_string(),
            text: "#1c1c1e".to_string(),
            muted_text: "#6e6e73".to_string(),
            accent: "#0a84ff".to_string(),
            accent_soft: "rgba(10, 132, 255, 0.12)".to_string(),
            accent_text: "#ffffff".to_string(),
            success: "#248a3d".to_string(),
            danger: "#c5281c".to_string(),
            canvas_base: "#ececf1".to_string(),
            grid_line: "rgba(28,28,30,0.09)".to_string(),
            selection_overlay: "rgba(0,0,0,0.14)".to_string(),
            shadow: "rgba(15,23,42,0.10)".to_string(),
        }
    }

    pub(crate) fn catppuccin_latte() -> Self {
        Self {
            name: "Catppuccin Latte".to_string(),
            appearance: ThemeAppearance::Light,
            background: "#eff1f5".to_string(),
            background_elevated: "#e6e9ef".to_string(),
            surface: "#ffffff".to_string(),
            surface_elevated: "#ccd0da".to_string(),
            surface_overlay: "rgba(255, 255, 255, 0.92)".to_string(),
            border: "#bcc0cc".to_string(),
            border_strong: "#acb0be".to_string(),
            text: "#4c4f69".to_string(),
            muted_text: "#5c5f77".to_string(),
            accent: "#1e66f5".to_string(),
            accent_soft: "rgba(30, 102, 245, 0.14)".to_string(),
            accent_text: "#ffffff".to_string(),
            success: "#40a02b".to_string(),
            danger: "#d20f39".to_string(),
            canvas_base: "#dce0e8".to_string(),
            grid_line: "rgba(76,79,105,0.10)".to_string(),
            selection_overlay: "rgba(76,79,105,0.12)".to_string(),
            shadow: "rgba(76,79,105,0.12)".to_string(),
        }
    }

    pub(crate) fn catppuccin_frappe() -> Self {
        Self {
            name: "Catppuccin Frappe".to_string(),
            appearance: ThemeAppearance::Dark,
            background: "#303446".to_string(),
            background_elevated: "#292c3c".to_string(),
            surface: "#414559".to_string(),
            surface_elevated: "#51576d".to_string(),
            surface_overlay: "rgba(48, 52, 70, 0.92)".to_string(),
            border: "#626880".to_string(),
            border_strong: "#737994".to_string(),
            text: "#c6d0f5".to_string(),
            muted_text: "#a5adce".to_string(),
            accent: "#8caaee".to_string(),
            accent_soft: "rgba(140, 170, 238, 0.16)".to_string(),
            accent_text: "#232634".to_string(),
            success: "#a6d189".to_string(),
            danger: "#e78284".to_string(),
            canvas_base: "#232634".to_string(),
            grid_line: "rgba(198,208,245,0.085)".to_string(),
            selection_overlay: "rgba(35,38,52,0.38)".to_string(),
            shadow: "rgba(0,0,0,0.30)".to_string(),
        }
    }

    pub(crate) fn catppuccin_macchiato() -> Self {
        Self {
            name: "Catppuccin Macchiato".to_string(),
            appearance: ThemeAppearance::Dark,
            background: "#24273a".to_string(),
            background_elevated: "#1e2030".to_string(),
            surface: "#363a4f".to_string(),
            surface_elevated: "#494d64".to_string(),
            surface_overlay: "rgba(36, 39, 58, 0.92)".to_string(),
            border: "#5b6078".to_string(),
            border_strong: "#6e738d".to_string(),
            text: "#cad3f5".to_string(),
            muted_text: "#a5adcb".to_string(),
            accent: "#8aadf4".to_string(),
            accent_soft: "rgba(138, 173, 244, 0.16)".to_string(),
            accent_text: "#181926".to_string(),
            success: "#a6da95".to_string(),
            danger: "#ed8796".to_string(),
            canvas_base: "#1e2030".to_string(),
            grid_line: "rgba(202,211,245,0.085)".to_string(),
            selection_overlay: "rgba(24,25,38,0.38)".to_string(),
            shadow: "rgba(0,0,0,0.30)".to_string(),
        }
    }

    pub(crate) fn catppuccin_mocha() -> Self {
        Self {
            name: "Catppuccin Mocha".to_string(),
            appearance: ThemeAppearance::Dark,
            background: "#1e1e2e".to_string(),
            background_elevated: "#181825".to_string(),
            surface: "#313244".to_string(),
            surface_elevated: "#45475a".to_string(),
            surface_overlay: "rgba(30, 30, 46, 0.92)".to_string(),
            border: "#585b70".to_string(),
            border_strong: "#6c7086".to_string(),
            text: "#cdd6f4".to_string(),
            muted_text: "#a6adc8".to_string(),
            accent: "#89b4fa".to_string(),
            accent_soft: "rgba(137, 180, 250, 0.16)".to_string(),
            accent_text: "#11111b".to_string(),
            success: "#a6e3a1".to_string(),
            danger: "#f38ba8".to_string(),
            canvas_base: "#181825".to_string(),
            grid_line: "rgba(205,214,244,0.085)".to_string(),
            selection_overlay: "rgba(17,17,27,0.40)".to_string(),
            shadow: "rgba(0,0,0,0.32)".to_string(),
        }
    }
}

pub(crate) fn default_custom_theme() -> ThemePalette {
    ThemePalette::taled_dark()
}

pub(crate) fn export_theme_json(theme: &ThemePalette) -> Result<String, String> {
    serde_json::to_string_pretty(theme).map_err(|error| error.to_string())
}

pub(crate) fn import_theme_json(source: &str) -> Result<ThemePalette, String> {
    let mut theme =
        serde_json::from_str::<ThemePalette>(source).map_err(|error| error.to_string())?;
    if theme.name.trim().is_empty() {
        theme.name = "Imported Theme".to_string();
    }
    Ok(theme)
}

pub(crate) fn resolved_theme(choice: ThemeChoice, custom: &ThemePalette) -> ThemePalette {
    match choice {
        ThemeChoice::System => match detect_system_theme() {
            ThemeAppearance::Dark => ThemePalette::taled_dark(),
            ThemeAppearance::Light => ThemePalette::taled_light(),
        },
        ThemeChoice::Dark => ThemePalette::taled_dark(),
        ThemeChoice::Light => ThemePalette::taled_light(),
        ThemeChoice::CatppuccinLatte => ThemePalette::catppuccin_latte(),
        ThemeChoice::CatppuccinFrappe => ThemePalette::catppuccin_frappe(),
        ThemeChoice::CatppuccinMacchiato => ThemePalette::catppuccin_macchiato(),
        ThemeChoice::CatppuccinMocha => ThemePalette::catppuccin_mocha(),
        ThemeChoice::Custom => custom.clone(),
    }
}

pub(crate) fn runtime_theme_css(choice: ThemeChoice, custom: &ThemePalette) -> String {
    let theme = resolved_theme(choice, custom);
    let color_scheme = match theme.appearance {
        ThemeAppearance::Light => "light",
        ThemeAppearance::Dark => "dark",
    };

    format!(
        r#"
        :root {{
          color-scheme: {color_scheme};
          --taled-theme-bg: {background};
          --taled-theme-bg-elevated: {background_elevated};
          --taled-theme-surface: {surface};
          --taled-theme-surface-elevated: {surface_elevated};
          --taled-theme-surface-overlay: {surface_overlay};
          --taled-theme-border: {border};
          --taled-theme-border-strong: {border_strong};
          --taled-theme-text: {text};
          --taled-theme-muted: {muted_text};
          --taled-theme-accent: {accent};
          --taled-theme-accent-soft: {accent_soft};
          --taled-theme-accent-text: {accent_text};
          --taled-theme-success: {success};
          --taled-theme-danger: {danger};
          --taled-theme-canvas: {canvas_base};
          --taled-theme-grid-line: {grid_line};
          --taled-theme-selection-overlay: {selection_overlay};
          --taled-theme-shadow: {shadow};
        }}
        "#,
        background = theme.background,
        background_elevated = theme.background_elevated,
        surface = theme.surface,
        surface_elevated = theme.surface_elevated,
        surface_overlay = theme.surface_overlay,
        border = theme.border,
        border_strong = theme.border_strong,
        text = theme.text,
        muted_text = theme.muted_text,
        accent = theme.accent,
        accent_soft = theme.accent_soft,
        accent_text = theme.accent_text,
        success = theme.success,
        danger = theme.danger,
        canvas_base = theme.canvas_base,
        grid_line = theme.grid_line,
        selection_overlay = theme.selection_overlay,
        shadow = theme.shadow,
    )
}

pub(crate) const THEME_STYLE_OVERRIDES: &str = r#"
  html, body, .app-shell {
    background: var(--taled-theme-bg) !important;
    color: var(--taled-theme-text) !important;
  }
  .mobile-shell.review-shell,
  .review-page,
  .review-body,
  .workspace {
    background: var(--taled-theme-bg) !important;
    color: var(--taled-theme-text) !important;
  }
  .topbar,
  .panel,
  .web-log-panel,
  .review-header,
  .review-bottom-nav,
  .review-project-list-panel,
  .review-create-project,
  .review-secondary-button,
  .review-sync-button,
  .review-project-row,
  .review-project-card,
  .review-info-card,
  .review-settings-card,
  .review-settings-card.single,
  .review-note-card,
  .review-about-hero,
  .review-settings-card.about-embedded,
  .review-property-field-card,
  .review-property-group-card,
  .review-field input,
  .review-property-field-value,
  .review-color-chip,
  .review-segmented,
  .review-select-input,
  .review-editor-toolbar,
  .review-tile-strip-top-shell,
  .review-tile-strip-live,
  .review-layer-float,
  .review-pan-joystick,
  .review-zoom-control,
  .review-history-button,
  .review-selection-actions,
  .review-license-card {
    background: var(--taled-theme-surface) !important;
    border-color: var(--taled-theme-border) !important;
    color: var(--taled-theme-text) !important;
  }
  .review-header,
  .review-bottom-nav,
  .topbar {
    background: var(--taled-theme-bg-elevated) !important;
  }
  .review-project-list-panel,
  .review-project-row,
  .review-settings-card.about-embedded,
  .review-field input,
  .review-select-input,
  .review-property-field-value,
  .review-color-chip,
  .review-segmented,
  .review-tile-strip-top-shell,
  .review-tile-strip-live {
    background: var(--taled-theme-surface-elevated) !important;
  }
  .review-pan-joystick,
  .review-zoom-control,
  .review-layer-float,
  .review-selection-actions,
  .review-history-button {
    background: var(--taled-theme-surface-overlay) !important;
    box-shadow: 0 10px 28px var(--taled-theme-shadow) !important;
  }
  .review-header h1,
  .review-project-title,
  .review-info-title,
  .review-layer-name,
  .review-about-link-title,
  .review-selected-tile-summary,
  .review-property-field-label,
  .review-zoom-control-label,
  .review-layer-float-name {
    color: var(--taled-theme-text) !important;
  }
  .review-project-meta,
  .review-info-meta,
  .review-sync-meta,
  .review-script-row,
  .muted,
  .review-caption,
  .review-setting-meta,
  .review-about-link-url,
  .review-disclosure-copy,
  .review-layer-float-current,
  .review-layer-float-kind,
  .review-menu-glyph,
  .review-eye,
  .review-lock,
  .review-tool,
  .review-tool-subbutton,
  .review-nav-item,
  .review-zoom-control-glyph {
    color: var(--taled-theme-muted) !important;
  }
  .review-link-button,
  .review-link,
  .review-about-link,
  .review-property-add-link,
  .review-nav-item.active,
  .review-eye.on,
  .review-lock.on {
    color: var(--taled-theme-accent) !important;
  }
  .review-tool.active,
  .review-tool-subbutton.active,
  .review-history-button,
  .review-selection-action {
    color: var(--taled-theme-text) !important;
  }
  .review-tool.active,
  .review-tool-subbutton.active,
  .review-layer-float-item.active {
    background: var(--taled-theme-accent-soft) !important;
  }
  .review-layer-float-item.active {
    box-shadow: inset 0 0 0 1px var(--taled-theme-border-strong) !important;
  }
  .review-history-button.disabled,
  .review-tool.placeholder,
  .review-tool-subbutton.placeholder,
  .review-tile-strip-side-empty {
    color: var(--taled-theme-muted) !important;
    opacity: 0.72;
  }
  .review-selection-action + .review-selection-action {
    border-left-color: var(--taled-theme-border) !important;
  }
  .review-tool-divider,
  .review-tile-strip-side-divider {
    background: var(--taled-theme-border) !important;
  }
  .review-tileset-sheet,
  .review-settings-card {
    background: var(--taled-theme-border) !important;
  }
  .review-sheet-cell,
  .review-tile-chip,
  .review-tile-chip.live {
    background-color: var(--taled-theme-bg-elevated) !important;
    border-color: var(--taled-theme-border) !important;
  }
  .review-tile-chip.selected {
    border-color: var(--taled-theme-accent) !important;
  }
  .review-sheet-cell.active {
    box-shadow: inset 0 0 0 3px var(--taled-theme-accent) !important;
  }
  .review-editor-canvas {
    background:
      linear-gradient(var(--taled-theme-grid-line) var(--grid-line-width), transparent var(--grid-line-width)),
      linear-gradient(90deg, var(--taled-theme-grid-line) var(--grid-line-width), transparent var(--grid-line-width)),
      var(--taled-theme-canvas) !important;
  }
  .review-map-live .cell-hitbox {
    border-color: var(--taled-theme-border) !important;
    opacity: 0.25;
  }
  .review-pan-joystick-ring {
    border-color: var(--taled-theme-border-strong) !important;
    background: transparent !important;
  }
  .review-pan-joystick-knob,
  .review-zoom-control-knob {
    background: color-mix(in srgb, var(--taled-theme-text) 12%, transparent) !important;
    border-color: color-mix(in srgb, var(--taled-theme-text) 16%, transparent) !important;
    box-shadow: 0 4px 18px var(--taled-theme-shadow) !important;
  }
  .review-zoom-control-track {
    background: color-mix(in srgb, var(--taled-theme-text) 8%, transparent) !important;
  }
  .review-toggle.on {
    background: var(--taled-theme-accent) !important;
  }
  .review-header-action:focus,
  .review-link-button:focus,
  .review-select-input:focus,
  .review-field input:focus {
    outline: none;
    border-color: var(--taled-theme-accent) !important;
    box-shadow: 0 0 0 2px var(--taled-theme-accent-soft) !important;
  }
"#;

#[cfg(target_arch = "wasm32")]
fn detect_system_theme() -> ThemeAppearance {
    use web_sys::window;

    window()
        .and_then(|window| window.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .map(|query| {
            if query.matches() {
                ThemeAppearance::Dark
            } else {
                ThemeAppearance::Light
            }
        })
        .unwrap_or(ThemeAppearance::Dark)
}

#[cfg(target_os = "android")]
fn detect_system_theme() -> ThemeAppearance {
    use jni::{JavaVM, objects::JObject};

    const UI_MODE_NIGHT_MASK: i32 = 0x30;
    const UI_MODE_NIGHT_YES: i32 = 0x20;

    let Some(appearance) = (|| {
        let context = ndk_context::android_context();
        let vm = unsafe { JavaVM::from_raw(context.vm().cast()) }.ok()?;
        let mut env = vm.attach_current_thread().ok()?;
        let android_context = unsafe { JObject::from_raw(context.context().cast()) };

        let resources = env
            .call_method(
                &android_context,
                "getResources",
                "()Landroid/content/res/Resources;",
                &[],
            )
            .ok()?
            .l()
            .ok()?;
        let configuration = env
            .call_method(
                resources,
                "getConfiguration",
                "()Landroid/content/res/Configuration;",
                &[],
            )
            .ok()?
            .l()
            .ok()?;
        let ui_mode = env.get_field(configuration, "uiMode", "I").ok()?.i().ok()?;

        Some(if (ui_mode & UI_MODE_NIGHT_MASK) == UI_MODE_NIGHT_YES {
            ThemeAppearance::Dark
        } else {
            ThemeAppearance::Light
        })
    })() else {
        return ThemeAppearance::Dark;
    };

    appearance
}

#[cfg(all(not(target_arch = "wasm32"), not(target_os = "android")))]
fn detect_system_theme() -> ThemeAppearance {
    ThemeAppearance::Dark
}

#[cfg(test)]
mod tests {
    use super::{ThemeAppearance, ThemePalette, export_theme_json, import_theme_json};

    #[test]
    fn theme_palette_json_round_trip() {
        let theme = ThemePalette::catppuccin_mocha();
        let exported = export_theme_json(&theme).expect("export theme");
        let imported = import_theme_json(&exported).expect("import theme");
        assert_eq!(imported, theme);
    }

    #[test]
    fn imported_theme_keeps_explicit_appearance() {
        let imported = import_theme_json(
            r##"{
              "name": "Smoke Test",
              "appearance": "light",
              "background": "#ffffff",
              "background_elevated": "#f8f8f8",
              "surface": "#ffffff",
              "surface_elevated": "#f0f0f0",
              "surface_overlay": "rgba(255,255,255,0.9)",
              "border": "#dddddd",
              "border_strong": "#cccccc",
              "text": "#111111",
              "muted_text": "#444444",
              "accent": "#3366ff",
              "accent_soft": "rgba(51,102,255,0.12)",
              "accent_text": "#ffffff",
              "success": "#22aa55",
              "danger": "#cc3344",
              "canvas_base": "#efefef",
              "grid_line": "rgba(0,0,0,0.08)",
              "selection_overlay": "rgba(0,0,0,0.12)",
              "shadow": "rgba(0,0,0,0.14)"
            }"##,
        )
        .expect("import theme");

        assert_eq!(imported.appearance, ThemeAppearance::Light);
        assert_eq!(imported.name, "Smoke Test");
    }
}
