use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen, Tool};
use crate::canvas::render_canvas;
use crate::l10n;
use crate::session_ops::adjust_zoom;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    render_editor_header(ui, state, theme);
    render_tile_strip_shell(ui, state, theme);

    // Canvas fills remaining space between tile strip and toolbar
    let header_h = 56.0;
    let strip_h = 114.0;
    let toolbar_h = 68.0;
    let nav_h = 72.0;
    let canvas_h = screen_height() - header_h - strip_h - toolbar_h - nav_h;
    let canvas_w = screen_width();

    render_canvas(ui, state, theme, canvas_w, canvas_h);

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
                        state.navigate(MobileScreen::Dashboard);
                    }
                    ui.text(&back, |t| {
                        t.font_size(14)
                            .color(super::widgets::HEADER_ACTION_COLOR)
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

            // Right: tool side panel (62px, mostly empty in default state)
            ui.element()
                .id("tool-side-panel")
                .width(fixed!(62.0))
                .height(grow!())
                .layout(|l| l.padding((8, 4, 8, 4)))
                .empty();
        });
}

struct PaletteTile {
    gid: u32,
    tileset_index: usize,
    local_id: u32,
}

fn collect_palette_preview(state: &AppState, limit: usize) -> Vec<PaletteTile> {
    let mut palette = Vec::with_capacity(limit);
    let Some(session) = state.session.as_ref() else {
        return palette;
    };
    for (tileset_index, tileset) in session.document().map.tilesets.iter().enumerate() {
        for local_id in 0..tileset.tileset.tile_count {
            palette.push(PaletteTile {
                gid: tileset.first_gid + local_id,
                tileset_index,
                local_id,
            });
            if palette.len() >= limit {
                return palette;
            }
        }
    }
    palette
}

/// Render tile chips in a 2-row column-first grid (matching CSS grid-auto-flow: column).
fn render_tile_chip_grid(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    palette: &[PaletteTile],
) {
    let num_cols = palette.len().div_ceil(2);

    // Pre-compute indices for each row (column-first: col*2+row)
    let row_indices: [Vec<usize>; 2] = [
        (0..num_cols).map(|c| c * 2).filter(|&i| i < palette.len()).collect(),
        (0..num_cols).map(|c| c * 2 + 1).filter(|&i| i < palette.len()).collect(),
    ];

    for indices in &row_indices {
        ui.element()
            .width(fit!())
            .height(fixed!(44.0))
            .layout(|l| l.direction(LeftToRight).align(Left, Top).gap(6))
            .children(|ui| {
                for &idx in indices {
                    render_tile_chip(ui, state, theme, &palette[idx]);
                }
            });
    }
}

fn render_tile_chip(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, tile: &PaletteTile) {
    let is_selected = state.selected_gid == tile.gid;
    let chip_bg = Color::u_rgb(0x10, 0x11, 0x13);
    let border_color = if is_selected {
        theme.accent
    } else {
        theme.border
    };
    let border_width = if is_selected { 2 } else { 1 };

    let tile_tex = crop_tile_texture(state, tile);
    let gid = tile.gid;

    ui.element()
        .id(("tile-chip", gid))
        .width(fixed!(44.0))
        .height(fixed!(44.0))
        .background_color(chip_bg)
        .corner_radius(8.0)
        .border(|b| b.all(border_width).color(border_color))
        .overflow(|o| o.clip())
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.selected_gid = gid;
            }
            if let Some(tex) = tile_tex {
                ui.element()
                    .width(fixed!(40.0))
                    .height(fixed!(40.0))
                    .image(tex)
                    .empty();
            }
        });
}

fn crop_tile_texture(state: &AppState, tile: &PaletteTile) -> Option<Texture2D> {
    let session = state.session.as_ref()?;
    let texture = state.tileset_textures.get(&tile.tileset_index)?;
    let tile_ref = session.document().map.tile_reference_for_gid(tile.gid)?;

    let ts = &tile_ref.tileset.tileset;
    let cols = ts.columns.max(1);
    let tw = ts.tile_width as f32;
    let th = ts.tile_height as f32;
    let sx = (tile.local_id % cols) as f32 * tw;
    let sy = (tile.local_id / cols) as f32 * th;

    // Render the cropped tile into a small texture
    let chip_size = 40.0;
    let scale = (chip_size / tw).min(chip_size / th);
    let rw = tw * scale;
    let rh = th * scale;
    let ox = (chip_size - rw) / 2.0;
    let oy = (chip_size - rh) / 2.0;

    let tex = render_to_texture(chip_size, chip_size, || {
        clear_background(MacroquadColor::from_rgba(0x10, 0x11, 0x13, 255));
        draw_texture_ex(
            texture,
            ox,
            oy,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(sx, sy, tw, th)),
                dest_size: Some(Vec2::new(rw, rh)),
                ..Default::default()
            },
        );
    });
    tex.set_filter(FilterMode::Nearest);
    Some(tex)
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render_floating_controls(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    // D-pad joystick (bottom-left, 92x92) — above canvas bottom
    ui.element()
        .id("dpad")
        .width(fixed!(92.0))
        .height(fixed!(92.0))
        .floating(|f| {
            f.anchor((Left, Bottom), (Left, Bottom))
                .offset((18.0, -18.0))
                .z_index(10)
        })
        .background_color(theme.surface_elevated)
        .corner_radius(46.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(CenterX, CenterY))
        .children(|ui| {
            // Simple D-pad cross
            ui.element()
                .width(fixed!(60.0))
                .height(fixed!(60.0))
                .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(4))
                .children(|ui| {
                    // Up
                    ui.element()
                        .id("dpad-up")
                        .width(fixed!(24.0))
                        .height(fixed!(14.0))
                        .layout(|l| l.align(Left, CenterY))
                        .on_press(move |_, _| {})
                        .children(|ui| {
                            if ui.just_released() {
                                state.camera_y -= 16.0;
                            }
                            ui.text("▲", |t| {
                                t.font_size(12).color(theme.muted_text).alignment(CenterX)
                            });
                        });

                    // Middle row (Left, Center, Right)
                    ui.element()
                        .width(fixed!(60.0))
                        .height(fixed!(18.0))
                        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY).gap(6))
                        .children(|ui| {
                            ui.element()
                                .id("dpad-left")
                                .width(fixed!(14.0))
                                .height(fixed!(18.0))
                                .layout(|l| l.align(Left, CenterY))
                                .on_press(move |_, _| {})
                                .children(|ui| {
                                    if ui.just_released() {
                                        state.camera_x -= 16.0;
                                    }
                                    ui.text("◀", |t| {
                                        t.font_size(12).color(theme.muted_text).alignment(CenterX)
                                    });
                                });

                            ui.element()
                                .width(fixed!(16.0))
                                .height(fixed!(16.0))
                                .background_color(theme.surface)
                                .corner_radius(8.0)
                                .empty();

                            ui.element()
                                .id("dpad-right")
                                .width(fixed!(14.0))
                                .height(fixed!(18.0))
                                .layout(|l| l.align(Left, CenterY))
                                .on_press(move |_, _| {})
                                .children(|ui| {
                                    if ui.just_released() {
                                        state.camera_x += 16.0;
                                    }
                                    ui.text("▶", |t| {
                                        t.font_size(12).color(theme.muted_text).alignment(CenterX)
                                    });
                                });
                        });

                    // Down
                    ui.element()
                        .id("dpad-down")
                        .width(fixed!(24.0))
                        .height(fixed!(14.0))
                        .layout(|l| l.align(Left, CenterY))
                        .on_press(move |_, _| {})
                        .children(|ui| {
                            if ui.just_released() {
                                state.camera_y += 16.0;
                            }
                            ui.text("▼", |t| {
                                t.font_size(12).color(theme.muted_text).alignment(CenterX)
                            });
                        });
                });
        });

    // Zoom control (bottom-right, 118x42)
    ui.element()
        .id("zoom-float")
        .width(fixed!(118.0))
        .height(fixed!(42.0))
        .floating(|f| {
            f.anchor((Right, Bottom), (Right, Bottom))
                .offset((-18.0, -18.0))
                .z_index(10)
        })
        .background_color(theme.surface_elevated)
        .corner_radius(21.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY))
        .children(|ui| {
            ui.element()
                .id("zoom-out")
                .width(fixed!(36.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
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
                .width(fixed!(46.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    let zoom_text = format!("{}%", state.zoom_percent);
                    ui.text(&zoom_text, |t| {
                        t.font_size(12).color(theme.muted_text).alignment(CenterX)
                    });
                });

            ui.element()
                .id("zoom-in")
                .width(fixed!(36.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
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

fn handle_tool_press(ui: &mut Ui, state: &mut AppState, tool: Tool) {
    if ui.just_released() {
        state.tool = tool;
    }
}

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

    // Toolbar bg matches reference surface color #1c1c1e
    let toolbar_bg = theme.surface;

    ui.element()
        .id("toolbar")
        .width(grow!())
        .height(fixed!(68.0))
        .background_color(toolbar_bg)
        .border(|b| b.top(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((6, 8, 6, 8))
                .gap(4)
        })
        .children(|ui| {
            for (i, (tool, label_key)) in tools.iter().enumerate() {
                let is_active = state.tool == *tool;
                let label = l10n::text(lang, label_key);
                let tool_val = *tool;
                let color = if is_active { theme.accent } else { theme.text };
                let bg = if is_active {
                    theme.accent_soft
                } else {
                    Color::rgba(0.0, 0.0, 0.0, 0.0)
                };

                let icon_id = crate::icons::tool_icon_id(label_key);
                let icon_tex = state.icon_cache.get(icon_id);

                ui.element()
                    .id(("tool", i as u32))
                    .width(grow!())
                    .height(grow!())
                    .background_color(bg)
                    .corner_radius(10.0)
                    .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(4))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        handle_tool_press(ui, state, tool_val);
                        ui.element()
                            .width(fixed!(22.0))
                            .height(fixed!(22.0))
                            .background_color(color)
                            .image(icon_tex)
                            .empty();
                        ui.text(&label, |t| t.font_size(10).color(color));
                    });
            }
        });
}
