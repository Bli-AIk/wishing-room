use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::icons::IconId;
use crate::l10n;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items, page_header};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let back = l10n::text(lang, "common-back");
    let done = l10n::text(lang, "common-done");
    page_header(
        ui,
        theme,
        &l10n::text(lang, "nav-layers"),
        Some((&back, MobileScreen::Editor)),
        Some((&done, MobileScreen::Layers)),
        state,
    );

    // Layer list body
    ui.element()
        .id("layers-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((12, 14, 8, 14)).gap(12))
        .overflow(|o| {
            o.scroll_y().scrollbar(|s| {
                s.width(3.0)
                    .thumb_color(theme.border_strong)
                    .track_color(theme.surface)
                    .hide_after_frames(120)
            })
        })
        .children(|ui| {
            add_layer_buttons(ui, state, theme);
            render_layer_list(ui, state, theme);
        });

    super::layer_dialogs::render_delete_dialog(ui, state, theme);

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Layers);
}

fn add_layer_buttons(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let add_tile = l10n::text(lang, "layer-add-tile");
    let add_obj = l10n::text(lang, "layer-add-object");
    ui.element()
        .id("layer-add-row")
        .width(grow!())
        .height(fixed!(36.0))
        .layout(|l| l.direction(LeftToRight).gap(10))
        .children(|ui| {
            add_btn(ui, state, theme, "add-tile-btn", &add_tile, false);
            add_btn(ui, state, theme, "add-obj-btn", &add_obj, true);
        });
}

fn add_btn(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    id: &'static str,
    label: &str,
    is_object: bool,
) {
    let icon_id = if is_object {
        IconId::LayerTypeObject
    } else {
        IconId::LayerTypeTile
    };
    let icon_tex = state.icon_cache.get(icon_id);
    ui.element()
        .id(id)
        .width(grow!())
        .height(grow!())
        .background_color(theme.surface)
        .corner_radius(10.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY).padding((0, 10, 0, 10)).gap(6))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released()
                && let Some(session) = state.session.as_mut()
            {
                let n = session.document().map.layers.len();
                let name = if is_object {
                    format!("Object {}", n + 1)
                } else {
                    format!("Tile {}", n + 1)
                };
                let idx = if is_object {
                    session.document_mut().map.add_object_layer(&name)
                } else {
                    session.document_mut().map.add_tile_layer(&name)
                };
                state.active_layer = idx;
                state.tiles_dirty = true;
                state.canvas_dirty = true;
            }
            ui.element()
                .width(fixed!(16.0))
                .height(fixed!(16.0))
                .background_color(theme.accent)
                .image(icon_tex)
                .empty();
            ui.text(label, |t| t.font_size(13).color(theme.text));
        });
}

fn render_layer_list(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let Some(session) = state.session.as_ref() else {
        ui.text("No map loaded", |t| t.font_size(14).color(theme.muted_text));
        return;
    };
    let lang = state.resolved_language();
    let layers: Vec<(usize, String, bool, bool, bool)> = session
        .document()
        .map
        .layers
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let vis = l.visible() && !state.hidden_layers.contains(&i);
            (
                i,
                if l.name().is_empty() {
                    format!("Layer {i}")
                } else {
                    l.name().to_string()
                },
                l.as_object().is_some(),
                vis,
                l.locked(),
            )
        })
        .collect();
    let layer_count = layers.len();

    // Sync text input on rename start
    if let Some(ri) = state.rename_layer_index
        && !state.rename_synced
    {
        if let Some((_, name, ..)) = layers.iter().find(|(i, ..)| *i == ri) {
            ui.set_text_value("layer-rename-input", name);
        }
        state.rename_synced = true;
    }

    // Apply rename when focus leaves the text input
    super::layer_dialogs::apply_rename_if_done(ui, state);

    for (i, display, is_obj, vis, locked) in layers.iter().rev() {
        let i = *i;
        let renaming = state.rename_layer_index == Some(i);
        let show_actions = state.layer_actions_row == Some(i) && !renaming;

        if renaming {
            super::layer_dialogs::layer_row_rename(ui, state, theme, i, display);
        } else if show_actions {
            super::layer_dialogs::layer_row_actions(ui, state, theme, i, display, lang);
        } else {
            layer_row_normal(ui, state, theme, i, display, *is_obj, *vis, *locked, layer_count);
        }
    }
}

fn layer_row_normal(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    i: usize,
    display: &str,
    is_obj: bool,
    vis: bool,
    locked: bool,
    layer_count: usize,
) {
    let is_active = state.active_layer == i;
    let bg = if is_active {
        theme.accent_soft
    } else {
        theme.surface
    };

    ui.element()
        .id(("layer-row", i as u32))
        .width(grow!())
        .height(fit!())
        .background_color(bg)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((14, 10, 14, 10))
                .gap(10)
        })
        .on_press(move |_, _| {})
        .children(|ui| {
            handle_row_swipe(ui, state, i);
            drag_handle(ui, state, theme, i, layer_count);
            type_icon(ui, state, theme, is_obj);
            layer_info(ui, state, theme, i, display, is_obj);
            eye_toggle(ui, state, theme, i, vis);
            lock_icon(ui, theme, locked);
        });
}

fn handle_row_swipe(ui: &mut Ui, state: &mut AppState, i: usize) {
    if ui.just_pressed() {
        let (mx, _) = macroquad::prelude::mouse_position();
        state.layer_swipe_start = Some((i, mx));
    }
    if !ui.just_released() {
        return;
    }
    let (mx, _) = macroquad::prelude::mouse_position();
    let Some((si, sx)) = state.layer_swipe_start.take() else {
        return;
    };
    if si == i && sx - mx > 40.0 {
        state.layer_actions_row = Some(i);
    } else if si == i {
        state.active_layer = i;
        state.layer_actions_row = None;
    }
    if state.rename_layer_index.is_some_and(|ri| ri != i) {
        state.rename_layer_index = None;
        state.rename_synced = false;
        state.rename_had_focus = false;
    }
}

fn drag_handle(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    i: usize,
    layer_count: usize,
) {
    ui.element()
        .id(("layer-drag", i as u32))
        .width(fixed!(30.0))
        .height(fixed!(44.0))
        .layout(|l| l.direction(TopToBottom).align(Left, CenterY).gap(2))
        .children(|ui| {
            drag_arrow_up(ui, state, theme, i, layer_count);
            drag_arrow_down(ui, state, theme, i);
        });
}

fn drag_arrow_up(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    i: usize,
    layer_count: usize,
) {
    if i + 1 < layer_count {
        ui.element()
            .id(("layer-up", i as u32))
            .width(fixed!(24.0))
            .height(fixed!(16.0))
            .layout(|l| l.align(Left, CenterY))
            .on_press(move |_, _| {})
            .children(|ui| {
                if ui.just_released()
                    && let Some(session) = state.session.as_mut()
                {
                    session.document_mut().map.swap_layers(i, i + 1);
                    state.active_layer = i + 1;
                    state.tiles_dirty = true;
                    state.canvas_dirty = true;
                }
                ui.text("▲", |t| t.font_size(10).color(theme.muted_text));
            });
    } else {
        ui.element()
            .width(fixed!(24.0))
            .height(fixed!(16.0))
            .empty();
    }
}

fn drag_arrow_down(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, i: usize) {
    if i > 0 {
        ui.element()
            .id(("layer-dn", i as u32))
            .width(fixed!(24.0))
            .height(fixed!(16.0))
            .layout(|l| l.align(Left, CenterY))
            .on_press(move |_, _| {})
            .children(|ui| {
                if ui.just_released()
                    && let Some(session) = state.session.as_mut()
                {
                    session.document_mut().map.swap_layers(i, i - 1);
                    state.active_layer = i - 1;
                    state.tiles_dirty = true;
                    state.canvas_dirty = true;
                }
                ui.text("▼", |t| t.font_size(10).color(theme.muted_text));
            });
    } else {
        ui.element()
            .width(fixed!(24.0))
            .height(fixed!(16.0))
            .empty();
    }
}

fn type_icon(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, is_obj: bool) {
    let icon_id = if is_obj {
        IconId::LayerTypeObject
    } else {
        IconId::LayerTypeTile
    };
    let icon_tex = state.icon_cache.get(icon_id);
    let tint = if is_obj { theme.accent } else { theme.text };
    ui.element()
        .width(fixed!(28.0))
        .height(fixed!(28.0))
        .background_color(tint)
        .image(icon_tex)
        .empty();
}

fn layer_info(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    i: usize,
    display: &str,
    is_obj: bool,
) {
    let lang = state.resolved_language();
    let kind = if is_obj {
        l10n::text(lang, "layer-object")
    } else {
        l10n::text(lang, "layer-tile")
    };
    ui.element()
        .id(("layer-info", i as u32))
        .width(grow!())
        .height(fit!())
        .layout(|l| l.direction(TopToBottom).gap(4))
        .children(|ui| {
            ui.text(display, |t| t.font_size(15).color(theme.text));
            ui.text(&kind, |t| t.font_size(12).color(theme.muted_text));
            opacity_bar(ui, theme);
        });
}

fn eye_toggle(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, i: usize, vis: bool) {
    let eye_id = if vis { IconId::EyeOn } else { IconId::EyeOff };
    let eye_c = if vis { theme.accent } else { theme.muted_text };
    let eye_tex = state.icon_cache.get(eye_id);
    ui.element()
        .width(fixed!(20.0))
        .height(fixed!(20.0))
        .background_color(eye_c)
        .image(eye_tex)
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                let now_hidden = if state.hidden_layers.contains(&i) {
                    state.hidden_layers.remove(&i);
                    false
                } else {
                    state.hidden_layers.insert(i);
                    true
                };
                state.last_eye_toggle = Some((i, now_hidden));
                state.tiles_dirty = true;
                state.canvas_dirty = true;
            }
        });
}

fn lock_icon(ui: &mut Ui, theme: &PlyTheme, locked: bool) {
    let lk_c = if locked {
        theme.accent
    } else {
        theme.muted_text
    };
    ui.text(if locked { "🔒" } else { "🔓" }, |t| {
        t.font_size(14).color(lk_c)
    });
}

fn opacity_bar(ui: &mut Ui, theme: &PlyTheme) {
    let pct = 100;
    ui.element()
        .width(grow!())
        .height(fixed!(14.0))
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY).gap(6))
        .children(|ui| {
            ui.element()
                .width(grow!())
                .height(fixed!(4.0))
                .background_color(theme.accent)
                .corner_radius(2.0)
                .empty();
            let label = format!("{pct}%");
            ui.text(&label, |t| t.font_size(11).color(theme.muted_text));
        });
}
