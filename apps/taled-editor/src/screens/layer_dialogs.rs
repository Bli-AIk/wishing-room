use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::l10n;
use crate::theme::PlyTheme;

/// Renders the delete-layer confirmation dialog overlay.
pub(crate) fn render_delete_dialog(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let Some(layer_idx) = state.delete_layer_pending else {
        return;
    };

    let lang = state.resolved_language();
    let layer_name = state
        .session
        .as_ref()
        .and_then(|s| s.document().map.layer(layer_idx))
        .map_or_else(|| format!("Layer {layer_idx}"), |l| l.name().to_string());
    let title = l10n::text(lang, "layer-delete-title");
    let message = l10n::text_with_args(lang, "layer-delete-message", &[("name", layer_name)]);
    let confirm_label = l10n::text(lang, "layer-delete-confirm");
    let cancel_label = l10n::text(lang, "layer-delete-cancel");

    let sw = screen_width();
    let sh = screen_height();

    // Backdrop
    ui.element()
        .id("del-layer-backdrop")
        .width(fixed!(sw))
        .height(fixed!(sh))
        .background_color(Color::u_rgba(0, 0, 0, 150))
        .floating(|f| f.attach_root().offset((0.0, 0.0)))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.delete_layer_pending = None;
            }
        });

    // Card
    let card_w: f32 = 280.0;
    let card_x = (sw - card_w) / 2.0;
    let card_y = sh * 0.3;
    ui.element()
        .id("del-layer-card")
        .width(fixed!(card_w))
        .background_color(theme.background_elevated)
        .corner_radius(12.0)
        .border(|b| b.all(1).color(theme.border))
        .floating(|f| f.attach_root().offset((card_x, card_y)))
        .layout(|l| l.direction(TopToBottom).padding((20, 20, 20, 20)).gap(12))
        .children(|ui| {
            ui.text(&title, |t| t.font_size(17).color(theme.text));
            ui.text(&message, |t| t.font_size(14).color(theme.muted_text));
            delete_confirm_btn(ui, state, theme, layer_idx, &confirm_label);
            delete_cancel_btn(ui, state, theme, &cancel_label);
        });
}

fn delete_confirm_btn(
    ui: &mut Ui,
    state: &mut AppState,
    _theme: &PlyTheme,
    layer_idx: usize,
    label: &str,
) {
    let danger = Color::u_rgb(0xE5, 0x3E, 0x3E);
    ui.element()
        .id("del-layer-confirm")
        .width(grow!())
        .height(fixed!(40.0))
        .background_color(danger)
        .corner_radius(8.0)
        .layout(|l| l.align(Left, CenterY))
        .on_press(|_, _| {})
        .children(|ui| {
            if ui.just_released()
                && let Some(session) = state.session.as_mut()
            {
                session.document_mut().map.remove_layer(layer_idx);
                if state.active_layer >= layer_idx && state.active_layer > 0 {
                    state.active_layer -= 1;
                }
                state.hidden_layers.remove(&layer_idx);
                state.tiles_dirty = true;
                state.canvas_dirty = true;
                state.delete_layer_pending = None;
                state.layer_actions_row = None;
            } else if ui.just_released() {
                state.delete_layer_pending = None;
                state.layer_actions_row = None;
            }
            ui.text(label, |t| {
                t.font_size(15)
                    .color(Color::u_rgb(0xff, 0xff, 0xff))
                    .alignment(CenterX)
            });
        });
}

fn delete_cancel_btn(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, label: &str) {
    ui.element()
        .id("del-layer-cancel")
        .width(grow!())
        .height(fixed!(40.0))
        .corner_radius(8.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(Left, CenterY))
        .on_press(|_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.delete_layer_pending = None;
            }
            ui.text(label, |t| {
                t.font_size(15).color(theme.text).alignment(CenterX)
            });
        });
}

pub(super) fn apply_rename_if_done(ui: &Ui, state: &mut AppState) {
    let Some(ri) = state.rename_layer_index else {
        return;
    };
    if !state.rename_synced {
        return;
    }
    let focused = ui
        .focused_element()
        .is_some_and(|f| f == Id::new("layer-rename-input"));
    if focused {
        state.rename_had_focus = true;
        return;
    }
    if !state.rename_had_focus {
        return;
    }
    let new_name = ui.get_text_value("layer-rename-input");
    if let Some(session) = state.session.as_mut()
        && ri < session.document().map.layers.len()
        && !new_name.is_empty()
    {
        *session.document_mut().map.layers[ri].name_mut() = new_name.to_string();
    }
    state.rename_layer_index = None;
    state.rename_synced = false;
    state.rename_had_focus = false;
}

pub(super) fn layer_row_rename(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    i: usize,
    _display: &str,
) {
    ui.element()
        .id(("layer-rename-row", i as u32))
        .width(grow!())
        .height(fixed!(48.0))
        .background_color(theme.accent_soft)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.accent))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((10, 14, 10, 14))
                .gap(8)
        })
        .children(|ui| {
            ui.element()
                .id("layer-rename-input")
                .width(grow!())
                .height(fixed!(28.0))
                .background_color(theme.surface)
                .corner_radius(6.0)
                .layout(|l| l.padding((0, 6, 0, 6)).align(Left, CenterY))
                .text_input(|t| {
                    t.font_size(14)
                        .text_color(theme.text)
                        .cursor_color(theme.text)
                        .max_length(64)
                })
                .empty();
            rename_confirm_btn(ui, state, theme, i);
            rename_cancel_btn(ui, state, theme, i);
        });
}

fn rename_confirm_btn(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, i: usize) {
    ui.element()
        .id(("rename-ok", i as u32))
        .width(fixed!(36.0))
        .height(fixed!(28.0))
        .background_color(theme.accent)
        .corner_radius(6.0)
        .layout(|l| l.align(Left, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                let new_name = ui.get_text_value("layer-rename-input");
                if let Some(session) = state.session.as_mut()
                    && i < session.document().map.layers.len()
                    && !new_name.is_empty()
                {
                    *session.document_mut().map.layers[i].name_mut() = new_name.to_string();
                }
                state.rename_layer_index = None;
                state.rename_synced = false;
                state.rename_had_focus = false;
            }
            ui.text("✓", |t| {
                t.font_size(16)
                    .color(Color::u_rgb(0xff, 0xff, 0xff))
                    .alignment(CenterX)
            });
        });
}

fn rename_cancel_btn(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, i: usize) {
    ui.element()
        .id(("rename-no", i as u32))
        .width(fixed!(36.0))
        .height(fixed!(28.0))
        .corner_radius(6.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(Left, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.rename_layer_index = None;
                state.rename_synced = false;
                state.rename_had_focus = false;
            }
            ui.text("✕", |t| {
                t.font_size(14).color(theme.muted_text).alignment(CenterX)
            });
        });
}

pub(super) fn layer_row_actions(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    i: usize,
    display: &str,
    lang: crate::l10n::SupportedLanguage,
) {
    let rename_label = l10n::text(lang, "layer-rename");
    let delete_label = l10n::text(lang, "layer-delete");
    ui.element()
        .id(("layer-act-row", i as u32))
        .width(grow!())
        .height(fit!())
        .background_color(theme.surface_elevated)
        .corner_radius(20.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((10, 10, 10, 10))
                .gap(8)
        })
        .children(|ui| {
            ui.element()
                .id(("layer-act-close", i as u32))
                .width(fixed!(32.0))
                .height(fixed!(32.0))
                .layout(|l| l.align(Left, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.layer_actions_row = None;
                    }
                    ui.text("✕", |t| t.font_size(18).color(theme.muted_text));
                });
            ui.element()
                .width(grow!())
                .height(fit!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    ui.text(display, |t| t.font_size(14).color(theme.text));
                });
            action_btn(
                ui,
                state,
                theme,
                ("act-rename", i as u32),
                &rename_label,
                false,
                i,
            );
            action_btn(
                ui,
                state,
                theme,
                ("act-delete", i as u32),
                &delete_label,
                true,
                i,
            );
        });
}

fn action_btn(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    id: (&str, u32),
    label: &str,
    is_delete: bool,
    layer_idx: usize,
) {
    let bg = if is_delete {
        Color::u_rgb(0xE5, 0x3E, 0x3E)
    } else {
        theme.accent
    };
    ui.element()
        .id(id)
        .width(fixed!(64.0))
        .height(fixed!(36.0))
        .background_color(bg)
        .corner_radius(8.0)
        .layout(|l| l.align(Left, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                if is_delete {
                    state.delete_layer_pending = Some(layer_idx);
                } else {
                    state.rename_layer_index = Some(layer_idx);
                    state.layer_actions_row = None;
                }
            }
            ui.text(label, |t| {
                t.font_size(13)
                    .color(Color::u_rgb(0xff, 0xff, 0xff))
                    .alignment(CenterX)
            });
        });
}
