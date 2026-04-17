use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::icons::IconId;
use crate::l10n;
use crate::platform;
use crate::theme::PlyTheme;

/// Import action submenu popup (rendered as a floating overlay).
pub(crate) fn import_menu_popup(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    if !state.show_import_menu {
        return;
    }

    let lang = state.resolved_language();
    let sw = screen_width();
    let sh = screen_height();

    // Backdrop
    ui.element()
        .id("import-menu-backdrop")
        .width(fixed!(sw))
        .height(fixed!(sh))
        .background_color(Color::u_rgba(0, 0, 0, 120))
        .floating(|f| f.attach_root().offset((0.0, 0.0)))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.show_import_menu = false;
            }
        });

    // Menu card
    let popup_w: f32 = 280.0;
    let popup_x = (sw - popup_w) / 2.0;
    let label = l10n::text(lang, "import-menu-workspace");

    ui.element()
        .id("import-menu-popup")
        .width(fixed!(popup_w))
        .height(fit!())
        .background_color(theme.surface_elevated)
        .corner_radius(16.0)
        .border(|b| b.all(1).color(theme.border))
        .floating(|f| f.attach_root().offset((popup_x, 140.0)))
        .layout(|l| l.direction(TopToBottom).padding((8, 12, 8, 12)))
        .children(|ui| {
            let icon = state.icon_cache.get(IconId::Import);
            ui.element()
                .id("import-opt-workspace")
                .width(grow!())
                .height(fixed!(48.0))
                .layout(|l| {
                    l.direction(LeftToRight)
                        .align(Left, CenterY)
                        .padding((14, 0, 14, 0))
                        .gap(10)
                })
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.show_import_menu = false;
                        platform::launch_directory_picker("workspace");
                    }
                    ui.element()
                        .width(fixed!(18.0))
                        .height(fixed!(18.0))
                        .background_color(theme.text)
                        .image(icon)
                        .empty();
                    ui.text(&label, |t| t.font_size(16).color(theme.text));
                });
        });
}
