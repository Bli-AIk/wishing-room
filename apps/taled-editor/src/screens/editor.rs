use ply_engine::prelude::*;

use crate::app_state::{
    AppState, MobileScreen, ShapeFillMode, TileSelectionMode, Tool, is_tile_selection_tool,
};
use crate::canvas::render_canvas;
use crate::l10n;
use crate::theme::PlyTheme;

use super::editor_toolbar::render_toolbar;
use super::tile_palette::{collect_palette_preview, render_tile_chip_grid};
use super::widgets::{bottom_nav, editor_nav_items};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    render_editor_header(ui, state, theme);
    render_tile_strip_shell(ui, state, theme);

    // Canvas fills remaining space between tile strip and toolbar
    render_canvas(ui, state, theme);

    render_toolbar(ui, state, theme);

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Editor);
}

fn render_editor_header(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let title = state
        .session
        .as_ref()
        .map(|s| {
            s.document()
                .file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        })
        .unwrap_or_else(|| "Tile Map Editor".to_string());

    ui.element()
        .id("editor-header")
        .width(grow!())
        .height(fixed!(56.0))
        .background_color(theme.background_elevated)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((20, 16, 16, 16))
                .gap(6)
        })
        .children(|ui| {
            // Left: Back button (92px)
            let back = l10n::text(state.resolved_language(), "common-back");
            ui.element()
                .id("editor-back")
                .width(fixed!(92.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.navigate_back_to(MobileScreen::Dashboard);
                    }
                    ui.text(&back, |t| {
                        t.font_size(14).color(super::widgets::HEADER_ACTION_COLOR)
                    });
                });

            // Center: title
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    ui.text(&title, |t| {
                        t.font_size(17).color(theme.text).alignment(CenterX)
                    });
                });

            // Right: Settings (92px)
            ui.element()
                .id("editor-settings")
                .width(fixed!(92.0))
                .height(grow!())
                .layout(|l| l.align(Right, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.navigate(MobileScreen::Settings);
                    }
                    let settings = l10n::text(state.resolved_language(), "nav-settings");
                    ui.text(&settings, |t| {
                        t.font_size(14).color(theme.muted_text).alignment(Right)
                    });
                });
        });
}

/// Tile strip shell — 114px, sits between header and canvas.
/// Contains palette area (left) + side divider + tool panel (right).
fn render_tile_strip_shell(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let strip_bg = theme.surface_elevated;
    let divider_color = Color::rgba(1.0, 1.0, 1.0, 0.10);

    // Collect palette tiles (up to 24)
    let palette = collect_palette_preview(state, 24);

    ui.element()
        .id("tile-strip-shell")
        .width(grow!())
        .height(fixed!(114.0))
        .background_color(strip_bg)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight))
        .children(|ui| {
            // Left: palette area with tile chip grid (2 rows, column-first flow)
            ui.element()
                .id("tile-palette")
                .width(grow!())
                .height(grow!())
                .overflow(|o| o.clip())
                .layout(|l| {
                    l.direction(TopToBottom)
                        .align(Left, Top)
                        .padding((10, 14, 10, 14))
                        .gap(6)
                })
                .children(|ui| {
                    render_tile_chip_grid(ui, state, theme, &palette);
                });

            // Vertical divider
            ui.element()
                .width(fixed!(1.0))
                .height(grow!())
                .layout(|l| l.padding((10, 0, 10, 0)))
                .children(|ui| {
                    ui.element()
                        .width(fixed!(1.0))
                        .height(grow!())
                        .background_color(divider_color)
                        .corner_radius(0.5)
                        .empty();
                });

            // Right: tool side panel (62px)
            render_tool_side_panel(ui, state, theme);
        });
}

fn render_tool_side_panel(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let selection_active = is_tile_selection_tool(state.tool);
    let shape_fill_active = state.tool == Tool::ShapeFill;
    let has_options = selection_active || shape_fill_active;

    if has_options {
        ui.element()
            .id("tool-side-panel")
            .width(fixed!(62.0))
            .height(grow!())
            .overflow(|o| o.scroll_y())
            .layout(|l| {
                l.direction(TopToBottom)
                    .padding((8, 4, 8, 4))
                    .gap(3)
                    .align(CenterX, Top)
            })
            .children(|ui| {
                if selection_active {
                    render_selection_modes(ui, state, theme, lang);
                } else {
                    render_shape_fill_modes(ui, state, theme, lang);
                }
            });
    } else {
        render_side_empty(ui, theme, lang);
    }
}

fn render_side_empty(ui: &mut Ui, theme: &PlyTheme, lang: l10n::SupportedLanguage) {
    let empty_color = Color::u_rgb(0x6e, 0x6e, 0x73);
    let _ = theme;
    let line1 = l10n::text(lang, "tile-strip-side-empty-line-1");
    let line2 = l10n::text(lang, "tile-strip-side-empty-line-2");
    let combined = format!("{line1}\n{line2}");
    // Strip is 114px; text block ≈ 26px; top padding 44 centres it vertically.
    ui.element()
        .id("tool-side-panel")
        .width(fixed!(62.0))
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((44, 0, 0, 0)))
        .children(|ui| {
            ui.text(&combined, |t| {
                t.font_size(9).color(empty_color).alignment(CenterX)
            });
        });
}

use crate::icons::IconId;

fn render_mode_button(
    ui: &mut Ui,
    state: &mut AppState,
    id: &'static str,
    label: &str,
    active: bool,
    icon_id: IconId,
) -> bool {
    let text_color = if active {
        Color::u_rgb(0xff, 0xff, 0xff)
    } else {
        Color::u_rgb(0xd1, 0xd1, 0xd6)
    };
    let bg = if active {
        Color::u_rgba(142, 142, 147, 46)
    } else {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    };
    let icon_tex = state.icon_cache.get(icon_id);
    let mut released = false;

    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(52.0))
        .background_color(bg)
        .corner_radius(9.0)
        .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(2))
        .on_press(move |_, _| {})
        .children(|ui| {
            released = ui.just_released();
            ui.element()
                .width(fixed!(22.0))
                .height(fixed!(22.0))
                .background_color(text_color)
                .image(icon_tex)
                .empty();
            ui.text(label, |t| t.font_size(9).color(text_color));
        });
    released
}

fn render_selection_modes(
    ui: &mut Ui,
    state: &mut AppState,
    _theme: &PlyTheme,
    lang: l10n::SupportedLanguage,
) {
    let modes: [(TileSelectionMode, &str, IconId, &'static str); 4] = [
        (
            TileSelectionMode::Replace,
            "selection-mode-replace",
            IconId::ModeSelReplace,
            "sel-replace",
        ),
        (
            TileSelectionMode::Add,
            "selection-mode-add",
            IconId::ModeSelAdd,
            "sel-add",
        ),
        (
            TileSelectionMode::Subtract,
            "selection-mode-subtract",
            IconId::ModeSelSubtract,
            "sel-sub",
        ),
        (
            TileSelectionMode::Intersect,
            "selection-mode-intersect",
            IconId::ModeSelIntersect,
            "sel-inter",
        ),
    ];
    for (mode, key, icon_id, id) in &modes {
        let active = state.tile_selection_mode == *mode;
        let label = l10n::text(lang, key);
        let mode_val = *mode;
        if render_mode_button(ui, state, id, &label, active, *icon_id) {
            state.tile_selection_mode = mode_val;
        }
    }
}

fn render_shape_fill_modes(
    ui: &mut Ui,
    state: &mut AppState,
    _theme: &PlyTheme,
    lang: l10n::SupportedLanguage,
) {
    let modes: [(ShapeFillMode, &str, IconId, &'static str); 2] = [
        (
            ShapeFillMode::Rectangle,
            "shape-fill-mode-rectangle",
            IconId::ModeRectangle,
            "shp-rect",
        ),
        (
            ShapeFillMode::Ellipse,
            "shape-fill-mode-ellipse",
            IconId::ModeEllipse,
            "shp-ellip",
        ),
    ];
    for (mode, key, icon_id, id) in &modes {
        let active = state.shape_fill_mode == *mode;
        let label = l10n::text(lang, key);
        let mode_val = *mode;
        if render_mode_button(ui, state, id, &label, active, *icon_id) {
            state.shape_fill_mode = mode_val;
        }
    }
}

// Toolbar and floating controls extracted to editor_toolbar module.
