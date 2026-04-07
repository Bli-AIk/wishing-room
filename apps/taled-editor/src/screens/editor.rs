use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen, Tool};
use crate::canvas::render_canvas;
use crate::l10n;
use crate::session_ops::adjust_zoom;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    render_editor_header(ui, state, theme);

    let canvas_w = screen_width();
    let canvas_h = screen_height() - 44.0 - 56.0 - 44.0 - 56.0;
    render_canvas(ui, state, theme, canvas_w, canvas_h);

    render_toolbar(ui, state, theme);
    render_palette_strip(ui, state, theme);

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
        .height(fixed!(44.0))
        .background_color(theme.background_elevated)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((0, 12, 0, 12))
        })
        .children(|ui| {
            let back = l10n::text(state.resolved_language(), "common-back");
            ui.element()
                .id("editor-back")
                .width(fixed!(50.0))
                .height(fixed!(32.0))
                .layout(|l| l.align(Left, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.navigate(MobileScreen::Dashboard);
                    }
                    ui.text(&back, |t| t.font_size(15).color(theme.accent));
                });

            ui.element()
                .width(grow!())
                .height(fixed!(32.0))
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text(&title, |t| {
                        t.font_size(17).color(theme.text).alignment(CenterX)
                    });
                });

            ui.element()
                .id("editor-settings")
                .width(fixed!(60.0))
                .height(fixed!(32.0))
                .layout(|l| l.align(Right, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.navigate(MobileScreen::Settings);
                    }
                    let settings = l10n::text(state.resolved_language(), "nav-settings");
                    ui.text(&settings, |t| {
                        t.font_size(15).color(theme.muted_text).alignment(Right)
                    });
                });
        });
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
fn render_toolbar(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let tools: [(Tool, &str); 6] = [
        (Tool::Hand, "tool-hand"),
        (Tool::Paint, "tool-stamp"),
        (Tool::Fill, "tool-fill"),
        (Tool::Erase, "tool-eraser"),
        (Tool::Select, "tool-rect-select"),
        (Tool::ShapeFill, "tool-shape-fill"),
    ];

    ui.element()
        .id("toolbar")
        .width(grow!())
        .height(fixed!(44.0))
        .background_color(theme.surface)
        .border(|b| b.top(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY))
        .children(|ui| {
            for (i, (tool, label_key)) in tools.iter().enumerate() {
                let is_active = state.tool == *tool;
                let label = l10n::text(lang, label_key);
                let tool_val = *tool;
                let color = if is_active { theme.accent } else { theme.text };
                let bg = if is_active {
                    theme.accent_soft
                } else {
                    theme.surface
                };

                ui.element()
                    .id(("tool", i as u32))
                    .width(grow!())
                    .height(grow!())
                    .background_color(bg)
                    .corner_radius(6.0)
                    .layout(|l| l.align(CenterX, CenterY).gap(2))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            state.tool = tool_val;
                        }
                        ui.text(&label, |t| t.font_size(10).color(color).alignment(CenterX));
                    });
            }

            // Zoom controls
            ui.element()
                .id("zoom-out")
                .width(fixed!(36.0))
                .height(grow!())
                .layout(|l| l.align(CenterX, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        adjust_zoom(state, -25);
                    }
                    ui.text("−", |t| {
                        t.font_size(18).color(theme.text).alignment(CenterX)
                    });
                });

            ui.element()
                .width(fixed!(48.0))
                .height(grow!())
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    let zoom_text = format!("{}%", state.zoom_percent);
                    ui.text(&zoom_text, |t| {
                        t.font_size(11).color(theme.muted_text).alignment(CenterX)
                    });
                });

            ui.element()
                .id("zoom-in")
                .width(fixed!(36.0))
                .height(grow!())
                .layout(|l| l.align(CenterX, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        adjust_zoom(state, 25);
                    }
                    ui.text("+", |t| {
                        t.font_size(18).color(theme.text).alignment(CenterX)
                    });
                });
        });
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
fn render_palette_strip(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    ui.element()
        .id("palette-strip")
        .width(grow!())
        .height(fixed!(56.0))
        .background_color(theme.surface)
        .border(|b| b.top(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .gap(4)
                .padding((4, 4, 4, 4))
        })
        .overflow(|o| o.scroll_x())
        .children(|ui| {
            let Some(session) = state.session.as_ref() else {
                return;
            };
            let map = &session.document().map;

            for (ts_idx, ts_ref) in map.tilesets.iter().enumerate() {
                if ts_ref.tileset.name == "collision" {
                    continue;
                }
                let Some(texture) = state.tileset_textures.get(&ts_idx) else {
                    continue;
                };
                let ts = &ts_ref.tileset;
                let cols = (ts.image.width / ts.tile_width).max(1);
                let tile_count = ts.tile_count.min(64);

                for local_id in 0..tile_count {
                    let gid = ts_ref.first_gid + local_id;
                    let src_col = local_id % cols;
                    let src_row = local_id / cols;
                    let sx = src_col as f32 * ts.tile_width as f32;
                    let sy = src_row as f32 * ts.tile_height as f32;

                    let tile_tex =
                        render_to_texture(ts.tile_width as f32, ts.tile_height as f32, || {
                            clear_background(MacroquadColor::from_rgba(0, 0, 0, 0));
                            draw_texture_ex(
                                texture,
                                0.0,
                                0.0,
                                WHITE,
                                DrawTextureParams {
                                    source: Some(Rect::new(
                                        sx,
                                        sy,
                                        ts.tile_width as f32,
                                        ts.tile_height as f32,
                                    )),
                                    dest_size: Some(Vec2::new(
                                        ts.tile_width as f32,
                                        ts.tile_height as f32,
                                    )),
                                    ..Default::default()
                                },
                            );
                        });

                    let is_selected = state.selected_gid == gid;
                    let border_color = if is_selected {
                        theme.accent
                    } else {
                        theme.border
                    };

                    ui.element()
                        .id(("palette-tile", gid))
                        .width(fixed!(48.0))
                        .height(fixed!(48.0))
                        .image(tile_tex)
                        .corner_radius(4.0)
                        .border(|b| b.all(if is_selected { 2 } else { 1 }).color(border_color))
                        .on_press(move |_, _| {})
                        .children(|ui| {
                            if ui.just_released() {
                                state.selected_gid = gid;
                            }
                        });
                }
                break; // Only show first non-collision tileset
            }
        });
}
