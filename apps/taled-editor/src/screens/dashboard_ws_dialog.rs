use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::icons::IconId;
use crate::l10n;
use crate::theme::PlyTheme;
use crate::workspace::{self, BUILTIN_WORKSPACE};

pub(crate) fn ws_trash_btn(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, ws_index: usize) {
    let icon_tex = state.icon_cache.get(IconId::Trash);
    ui.element()
        .id(("ws-trash", ws_index as u32))
        .width(fixed!(28.0))
        .height(fixed!(28.0))
        .background_color(theme.danger)
        .corner_radius(6.0)
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.delete_workspace_pending = Some(ws_index);
                state.show_workspace_picker = false;
            }
            ui.element()
                .width(fixed!(16.0))
                .height(fixed!(16.0))
                .background_color(Color::u_rgb(0xff, 0xff, 0xff))
                .image(icon_tex)
                .empty();
        });
}

pub(crate) fn ws_delete_dialog(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let Some(idx) = state.delete_workspace_pending else {
        return;
    };
    let ws_name = state.workspace_list.get(idx).cloned().unwrap_or_default();
    if ws_name.is_empty() || ws_name == BUILTIN_WORKSPACE {
        state.delete_workspace_pending = None;
        return;
    }
    let lang = state.resolved_language();
    let title = l10n::text(lang, "workspace-delete-title");
    let message = l10n::text_with_args(
        lang,
        "workspace-delete-message",
        &[("name", ws_name.clone())],
    );
    let confirm_label = l10n::text(lang, "workspace-delete-confirm");
    let cancel_label = l10n::text(lang, "workspace-delete-cancel");
    let sw = screen_width();
    let sh = screen_height();

    // Backdrop
    ui.element()
        .id("del-ws-backdrop")
        .width(fixed!(sw))
        .height(fixed!(sh))
        .background_color(Color::u_rgba(0, 0, 0, 150))
        .floating(|f| f.attach_root().offset((0.0, 0.0)))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.delete_workspace_pending = None;
            }
        });

    // Card
    let card_w: f32 = 280.0;
    let card_x = (sw - card_w) / 2.0;
    let card_y = sh * 0.3;
    ui.element()
        .id("del-ws-card")
        .width(fixed!(card_w))
        .background_color(theme.background_elevated)
        .corner_radius(12.0)
        .border(|b| b.all(1).color(theme.border))
        .floating(|f| f.attach_root().offset((card_x, card_y)))
        .layout(|l| l.direction(TopToBottom).padding((20, 20, 20, 20)).gap(12))
        .children(|ui| {
            ui.text(&title, |t| t.font_size(17).color(theme.text));
            ui.text(&message, |t| t.font_size(14).color(theme.muted_text));
            ws_delete_confirm_btn(ui, state, theme, &ws_name, &confirm_label);
            ws_delete_cancel_btn(ui, state, theme, &cancel_label);
        });
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI confirm button with nested closures
fn ws_delete_confirm_btn(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    ws_name: &str,
    label: &str,
) {
    let name = ws_name.to_string();
    ui.element()
        .id("ws-del-confirm")
        .width(grow!())
        .height(fixed!(40.0))
        .background_color(theme.danger)
        .corner_radius(10.0)
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                if workspace::delete_workspace(&name) {
                    let lang = state.resolved_language();
                    state.workspace_list = workspace::list_workspaces()
                        .into_iter()
                        .map(|w| w.name)
                        .collect();
                    if state.active_workspace == name {
                        state.active_workspace = BUILTIN_WORKSPACE.to_string();
                    }
                    state.status = l10n::text(lang, "dashboard-workspace-deleted");
                    crate::thumbnails::invalidate_cache();
                }
                state.delete_workspace_pending = None;
            }
            ui.text(label, |t| {
                t.font_size(15).color(Color::u_rgb(0xff, 0xff, 0xff))
            });
        });
}

fn ws_delete_cancel_btn(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, label: &str) {
    ui.element()
        .id("ws-del-cancel")
        .width(grow!())
        .height(fixed!(40.0))
        .background_color(theme.surface)
        .corner_radius(10.0)
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.delete_workspace_pending = None;
            }
            ui.text(label, |t| t.font_size(15).color(theme.text));
        });
}
