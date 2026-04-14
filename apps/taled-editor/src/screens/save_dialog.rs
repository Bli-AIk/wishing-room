use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::l10n;
use crate::theme::PlyTheme;

/// Renders the unsaved-changes dialog overlay.
///
/// Returns early if `state.show_save_dialog` is false.
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    if !state.show_save_dialog {
        return;
    }

    let lang = state.resolved_language();
    let title = l10n::text(lang, "save-dialog-title");
    let message = l10n::text(lang, "save-dialog-message");
    let save_label = l10n::text(lang, "save-dialog-save");
    let discard_label = l10n::text(lang, "save-dialog-discard");
    let later_label = l10n::text(lang, "save-dialog-later");

    let sw = screen_width();
    let sh = screen_height();

    // Full-screen backdrop
    ui.element()
        .id("save-dialog-backdrop")
        .width(fixed!(sw))
        .height(fixed!(sh))
        .background_color(Color::u_rgba(0, 0, 0, 150))
        .floating(|f| f.attach_root().offset((0.0, 0.0)))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.show_save_dialog = false;
            }
        });

    // Dialog card centered on screen
    let card_w: f32 = 280.0;
    let card_x = (sw - card_w) / 2.0;
    let card_y = sh * 0.3;
    ui.element()
        .id("save-dialog-card")
        .width(fixed!(card_w))
        .background_color(theme.background_elevated)
        .corner_radius(12.0)
        .border(|b| b.all(1).color(theme.border))
        .floating(|f| f.attach_root().offset((card_x, card_y)))
        .layout(|l| l.direction(TopToBottom).padding((20, 20, 20, 20)).gap(12))
        .children(|ui| {
            ui.text(&title, |t| t.font_size(17).color(theme.text));
            ui.text(&message, |t| t.font_size(14).color(theme.muted_text));

            save_btn(ui, state, theme, &save_label);
            discard_btn(ui, state, theme, &discard_label);
            later_btn(ui, state, theme, &later_label);
        });
}

fn save_btn(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, label: &str) {
    ui.element()
        .id("save-dlg-save")
        .width(grow!())
        .height(fixed!(40.0))
        .background_color(theme.accent)
        .corner_radius(8.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(Left, CenterY))
        .on_press(|_, _| {})
        .children(|ui| {
            if ui.just_released() {
                if let Some(session) = state.session.as_mut()
                    && let Err(e) = session.save()
                {
                    crate::logging::append(&format!("save FAILED: {e}"));
                }
                state.show_save_dialog = false;
                state.navigate_back_to(MobileScreen::Dashboard);
            }
            ui.text(label, |t| {
                t.font_size(15)
                    .color(Color::u_rgb(0xff, 0xff, 0xff))
                    .alignment(CenterX)
            });
        });
}

fn discard_btn(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, label: &str) {
    ui.element()
        .id("save-dlg-discard")
        .width(grow!())
        .height(fixed!(40.0))
        .corner_radius(8.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(Left, CenterY))
        .on_press(|_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.show_save_dialog = false;
                state.navigate_back_to(MobileScreen::Dashboard);
            }
            ui.text(label, |t| {
                t.font_size(15).color(theme.muted_text).alignment(CenterX)
            });
        });
}

fn later_btn(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, label: &str) {
    ui.element()
        .id("save-dlg-later")
        .width(grow!())
        .height(fixed!(40.0))
        .corner_radius(8.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(Left, CenterY))
        .on_press(|_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.show_save_dialog = false;
            }
            ui.text(label, |t| {
                t.font_size(15).color(theme.text).alignment(CenterX)
            });
        });
}
