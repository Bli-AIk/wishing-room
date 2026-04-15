use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::icons::IconId;
use crate::l10n;
use crate::theme::PlyTheme;

use super::tile_palette::{PaletteTile, crop_tile_texture};
use super::widgets::{HEADER_ACTION_COLOR, bottom_nav, editor_nav_items};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    tileset_header(ui, state, theme);

    handle_sheet_pinch(state);

    let active_ts = state.active_tileset;
    ui.element()
        .id("tilesets-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((8, 14, 0, 14)).gap(6))
        .children(|ui| {
            let tile_info = state.session.as_ref().map(|session| {
                let map = &session.document().map;
                let Some(ts) = map.tilesets.get(active_ts) else {
                    return (1u32, vec![], 1u32, 16u32, 16u32, String::new(), false);
                };
                let (fg, tw, th) = (ts.first_gid, ts.tileset.tile_width, ts.tileset.tile_height);
                let name = ts.tileset.name.clone();
                let coi = ts.tileset.columns == 0 && !ts.tileset.tile_images.is_empty();
                if coi {
                    let mut ids: Vec<u32> = ts.tileset.tile_images.keys().copied().collect();
                    ids.sort();
                    let c = (ids.len() as f32).sqrt().ceil().max(1.0) as u32;
                    (c, ids, fg, tw, th, name, true)
                } else {
                    let c = ts.tileset.columns.max(1);
                    (c, (0..ts.tileset.tile_count).collect(), fg, tw, th, name, false)
                }
            });

            let Some((cols, tile_ids, first_gid, tw, th, ts_name, is_coi)) = tile_info else {
                ui.text("Load a TMX sample to view tilesets.", |t| {
                    t.font_size(14).color(theme.muted_text)
                });
                return;
            };

            sprite_sheet_view(ui, state, cols, &tile_ids, first_gid, tw, th);

            let sel_local = state.selected_gid.saturating_sub(first_gid);
            property_section(ui, state, theme, sel_local, &ts_name, tw, th, cols, is_coi);
        });

    tileset_picker(ui, state, theme);

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Tilesets);
}

fn tileset_header(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let back = l10n::text(lang, "common-back");
    let ts_name = state
        .session
        .as_ref()
        .and_then(|s| s.document().map.tilesets.get(state.active_tileset))
        .map(|t| t.tileset.name.clone())
        .unwrap_or_else(|| l10n::text(lang, "nav-tilesets"));

    ui.element()
        .id("ts-header")
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
            // Back button
            ui.element()
                .id("ts-back")
                .width(fixed!(92.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.navigate_down(MobileScreen::Editor);
                    }
                    ui.text(&back, |t| t.font_size(14).color(HEADER_ACTION_COLOR));
                });

            // Center: TSX name + chevron (tappable to open picker)
            ui.element()
                .id("ts-title")
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(CenterX, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.show_tileset_picker = !state.show_tileset_picker;
                    }
                    ui.element()
                        .width(fit!())
                        .height(fit!())
                        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY).gap(6))
                        .children(|ui| {
                            ui.text(&ts_name, |t| t.font_size(17).color(theme.text));
                            let ch = state.icon_cache.get(IconId::ChevronDown);
                            ui.element()
                                .width(fixed!(14.0))
                                .height(fixed!(14.0))
                                .background_color(theme.muted_text)
                                .image(ch)
                                .empty();
                        });
                });

            // Right spacer
            ui.element().width(fixed!(92.0)).height(fixed!(1.0)).empty();
        });
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI popup requires nested closures
fn tileset_picker(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    if !state.show_tileset_picker {
        return;
    }
    let lang = state.resolved_language();
    let sw = screen_width();
    let sh = screen_height();

    // Semi-transparent backdrop
    ui.element()
        .id("ts-picker-backdrop")
        .width(fixed!(sw))
        .height(fixed!(sh))
        .background_color(Color::u_rgba(0, 0, 0, 120))
        .floating(|f| f.attach_root().offset((0.0, 0.0)))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.show_tileset_picker = false;
            }
        });

    let popup_w: f32 = 300.0;
    let popup_x = (sw - popup_w) / 2.0;
    let tilesets: Vec<(usize, String)> = state
        .session
        .as_ref()
        .map(|s| {
            s.document()
                .map
                .tilesets
                .iter()
                .enumerate()
                .map(|(i, t)| (i, t.tileset.name.clone()))
                .collect()
        })
        .unwrap_or_default();

    ui.element()
        .id("ts-picker-popup")
        .width(fixed!(popup_w))
        .height(fit!())
        .background_color(theme.surface_elevated)
        .corner_radius(16.0)
        .border(|b| b.all(1).color(theme.border))
        .floating(|f| f.attach_root().offset((popup_x, 66.0)))
        .layout(|l| l.direction(TopToBottom).padding((8, 12, 8, 12)))
        .children(|ui| {
            for (idx, name) in &tilesets {
                let is_active = *idx == state.active_tileset;
                let text_color = if is_active { theme.accent } else { theme.text };
                let i = *idx;
                ui.element()
                    .id(("ts-item", i as u32))
                    .width(grow!())
                    .height(fixed!(44.0))
                    .layout(|l| l.align(Left, CenterY).padding((14, 0, 14, 0)))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            state.active_tileset = i;
                            state.show_tileset_picker = false;
                            state.viewfinder_offset = (0.0, 0.0);
                            if let Some(s) = state.session.as_ref()
                                && let Some(ts) = s.document().map.tilesets.get(i)
                            {
                                state.selected_gid = ts.first_gid;
                            }
                        }
                        ui.text(name, |t| t.font_size(16).color(text_color));
                    });
            }
            // "+ New Tileset" item
            let new_label = l10n::text(lang, "tileset-new");
            ui.element()
                .id("ts-new")
                .width(grow!())
                .height(fixed!(44.0))
                .layout(|l| l.align(Left, CenterY).padding((14, 0, 14, 0)))
                .children(|ui| {
                    ui.text(&new_label, |t| t.font_size(16).color(theme.muted_text));
                });
        });
}

fn handle_sheet_pinch(state: &mut AppState) {
    let all_touches = touches();
    if all_touches.len() >= 2 {
        let t0 = all_touches[0].position;
        let t1 = all_touches[1].position;
        let dx = t1.x - t0.x;
        let dy = t1.y - t0.y;
        let dist = (dx * dx + dy * dy).sqrt() as f64;
        if dist > 10.0 {
            if let Some(prev) = state.sheet_pinch_dist {
                let ratio = dist / prev;
                state.sheet_zoom = (state.sheet_zoom * ratio as f32).clamp(0.3, 10.0);
            }
            state.sheet_pinch_dist = Some(dist);
        }
    } else {
        state.sheet_pinch_dist = None;
    }
}

fn sprite_sheet_view(
    ui: &mut Ui,
    state: &mut AppState,
    cols: u32,
    tile_ids: &[u32],
    first_gid: u32,
    tw: u32,
    th: u32,
) {
    let avail_w = screen_width() - 28.0;
    let sheet_w = cols as f32 * tw as f32;
    let fit_zoom = avail_w / sheet_w;
    let sheet_key = cols * 10000 + tw;
    if state.sheet_zoom <= 0.0 || state.sheet_zoom_key != sheet_key {
        state.sheet_zoom = fit_zoom;
        state.sheet_zoom_key = sheet_key;
    }

    let zoom = state.sheet_zoom;
    let cell_w = tw as f32 * zoom;
    let cell_h = th as f32 * zoom;
    let count = tile_ids.len() as u32;
    let rows = count.div_ceil(cols);
    let is_pinching = touches().len() >= 2;

    ui.element()
        .id("sprite-sheet")
        .width(grow!())
        .height(grow!())
        .background_color(Color::from(0x101113_u32))
        .corner_radius(14.0)
        .layout(|l| l.direction(TopToBottom))
        .overflow(|o| o.scroll_x().scroll_y().clip())
        .children(|ui| {
            for row in 0..rows {
                sheet_row(
                    ui, state, row, cols, tile_ids, first_gid, cell_w, cell_h, is_pinching,
                );
            }
        });
}

fn sheet_row(
    ui: &mut Ui,
    state: &mut AppState,
    row: u32,
    cols: u32,
    tile_ids: &[u32],
    first_gid: u32,
    cell_w: f32,
    cell_h: f32,
    is_pinching: bool,
) {
    ui.element()
        .id(("sheet-row", row))
        .width(fit!())
        .height(fixed!(cell_h))
        .layout(|l| l.direction(LeftToRight))
        .children(|ui| {
            for col in 0..cols {
                let idx = (row * cols + col) as usize;
                let Some(&local_id) = tile_ids.get(idx) else {
                    break;
                };
                sheet_cell(
                    ui, state, first_gid + local_id, first_gid, cell_w, cell_h, is_pinching,
                );
            }
        });
}

fn sheet_cell(
    ui: &mut Ui,
    state: &mut AppState,
    gid: u32,
    first_gid: u32,
    w: f32,
    h: f32,
    is_pinching: bool,
) {
    let is_selected = state.selected_gid == gid;
    let local_id = gid - first_gid;
    let tile = PaletteTile {
        gid,
        tileset_index: state.active_tileset,
        local_id,
    };
    let tile_tex = crop_tile_texture(state, &tile);
    let has_tile = tile_tex.is_some();
    let sel_color = Color::u_rgba(255, 59, 48, 255);

    ui.element()
        .id(("cell", gid))
        .width(fixed!(w))
        .height(fixed!(h))
        .overflow(|o| o.clip())
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() && !is_pinching && has_tile {
                state.selected_gid = gid;
            }
            if let Some(tex) = tile_tex {
                ui.element()
                    .width(grow!())
                    .height(grow!())
                    .image(tex)
                    .empty();
            }
            if is_selected && has_tile {
                // Inset selection border overlay
                ui.element()
                    .id(("sel-border", gid))
                    .width(fixed!(w))
                    .height(fixed!(h))
                    .floating(|f| f.attach_parent().offset((0.0, 0.0)))
                    .border(|b| b.all(2).color(sel_color))
                    .empty();
            }
        });
}

fn property_section(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    local_id: u32,
    ts_name: &str,
    tw: u32,
    th: u32,
    cols: u32,
    is_coi: bool,
) {
    let expanded = state.property_panel_expanded;
    let arrow = if expanded { "▼" } else { "▲" };

    ui.element()
        .id("panel-toggle")
        .width(grow!())
        .height(fixed!(30.0))
        .background_color(theme.surface)
        .corner_radius(8.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(CenterX, CenterY).direction(LeftToRight).gap(6))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.property_panel_expanded = !state.property_panel_expanded;
            }
            ui.text(arrow, |t| t.font_size(11).color(theme.muted_text));
            ui.text("Property Panel", |t| {
                t.font_size(13).color(theme.muted_text)
            });
        });

    if !expanded {
        return;
    }

    let (x, y) = if is_coi {
        (0, 0)
    } else {
        ((local_id % cols) * tw, (local_id / cols) * th)
    };

    ui.element()
        .id("prop-panel")
        .width(grow!())
        .height(fixed!(260.0))
        .background_color(theme.surface)
        .corner_radius(14.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight).padding((10, 12, 10, 12)).gap(8))
        .children(|ui| {
            general_column(ui, theme, local_id, ts_name, x, y, tw, th, is_coi);
            ui.element()
                .width(fixed!(1.0))
                .height(grow!())
                .background_color(theme.border)
                .empty();
            custom_column(ui, theme);
        });
}

fn general_column(
    ui: &mut Ui,
    theme: &PlyTheme,
    local_id: u32,
    ts_name: &str,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    is_coi: bool,
) {
    ui.element()
        .id("general-col")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(5))
        .children(|ui| {
            ui.text("General", |t| t.font_size(14).color(theme.text));
            prop_row(ui, theme, "ID", &local_id.to_string());
            prop_row(ui, theme, "Tileset", ts_name);
            if !is_coi {
                prop_row(ui, theme, "X", &x.to_string());
                prop_row(ui, theme, "Y", &y.to_string());
                prop_row(ui, theme, "W", &w.to_string());
                prop_row(ui, theme, "H", &h.to_string());
            }
        });
}

fn custom_column(ui: &mut Ui, theme: &PlyTheme) {
    ui.element()
        .id("custom-col")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(5))
        .children(|ui| {
            ui.text("Custom Properties", |t| t.font_size(14).color(theme.text));
            ui.text("No properties", |t| t.font_size(12).color(theme.muted_text));
            ui.element()
                .id("add-prop-btn")
                .width(grow!())
                .height(fixed!(28.0))
                .corner_radius(8.0)
                .border(|b| b.all(1).color(theme.accent))
                .layout(|l| l.align(CenterX, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    ui.text("+ Add Property", |t| t.font_size(13).color(theme.accent));
                });
        });
}

fn prop_row(ui: &mut Ui, theme: &PlyTheme, label: &str, value: &str) {
    ui.element()
        .width(grow!())
        .height(fixed!(24.0))
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY).gap(4))
        .children(|ui| {
            ui.text(label, |t| t.font_size(11).color(theme.muted_text));
            ui.element()
                .width(grow!())
                .height(fixed!(20.0))
                .background_color(theme.border)
                .corner_radius(5.0)
                .layout(|l| l.padding((0, 6, 0, 6)).align(Left, CenterY))
                .children(|ui| {
                    ui.text(value, |t| t.font_size(11).color(theme.text));
                });
        });
}
