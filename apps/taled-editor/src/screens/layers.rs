use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::icons::IconId;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items, page_header};

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    page_header(
        ui,
        theme,
        "Layer Manager",
        Some(("Back", MobileScreen::Editor)),
        Some(("Done", MobileScreen::Layers)),
        state,
    );

    // Layer list
    ui.element()
        .id("layers-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((12, 14, 8, 14)))
        .overflow(|o| {
            o.scroll_y().scrollbar(|s| {
                s.width(3.0)
                    .thumb_color(theme.border_strong)
                    .track_color(theme.surface)
                    .hide_after_frames(120)
            })
        })
        .children(|ui| {
            let Some(session) = state.session.as_ref() else {
                ui.text("No map loaded", |t| t.font_size(14).color(theme.muted_text));
                return;
            };
            let map = &session.document().map;

            // Layer panel (rounded container)
            ui.element()
                .id("layer-panel")
                .width(grow!())
                .height(fit!())
                .background_color(theme.surface_elevated)
                .corner_radius(14.0)
                .border(|b| b.all(1).color(theme.border))
                .layout(|l| l.direction(TopToBottom))
                .children(|ui| {
                    for (i, layer) in map.layers.iter().enumerate() {
                        let is_active = state.active_layer == i;
                        let is_first = i == 0;

                        let bg = if is_active {
                            theme.accent_soft
                        } else {
                            theme.surface_elevated
                        };

                        ui.element()
                            .id(("layer-row", i as u32))
                            .width(grow!())
                            .height(fixed!(72.0))
                            .background_color(bg)
                            .border(|b| {
                                if is_first {
                                    b
                                } else {
                                    b.top(1).color(theme.border)
                                }
                            })
                            .layout(|l| {
                                l.direction(LeftToRight)
                                    .align(Left, CenterY)
                                    .padding((10, 14, 10, 14))
                                    .gap(10)
                            })
                            .on_press(move |_, _| {})
                            .children(|ui| {
                                if ui.just_released() {
                                    state.active_layer = i;
                                }

                                // Drag handle
                                ui.text("≡", |t| t.font_size(20).color(theme.muted_text));

                                // Layer thumbnail (32x32)
                                ui.element()
                                    .id(("layer-thumb", i as u32))
                                    .width(fixed!(32.0))
                                    .height(fixed!(32.0))
                                    .background_color(theme.surface)
                                    .corner_radius(6.0)
                                    .border(|b| b.all(1).color(theme.border))
                                    .empty();

                                // Layer info (name + opacity)
                                ui.element()
                                    .width(grow!())
                                    .height(fit!())
                                    .layout(|l| l.direction(TopToBottom).gap(4))
                                    .children(|ui| {
                                        let name = layer.name();
                                        let display = if name.is_empty() {
                                            format!("Layer {}", i)
                                        } else {
                                            name.to_string()
                                        };
                                        ui.text(&display, |t| t.font_size(15).color(theme.text));

                                        // Opacity bar (hardcoded: model has no opacity field yet)
                                        let pct = 100;
                                        ui.element()
                                            .width(grow!())
                                            .height(fixed!(14.0))
                                            .layout(|l| {
                                                l.direction(LeftToRight).align(Left, CenterY).gap(6)
                                            })
                                            .children(|ui| {
                                                // Opacity track (full width = 100%)
                                                ui.element()
                                                    .width(grow!())
                                                    .height(fixed!(4.0))
                                                    .background_color(theme.accent)
                                                    .corner_radius(2.0)
                                                    .empty();

                                                let label = format!("{}%", pct);
                                                ui.text(&label, |t| {
                                                    t.font_size(11).color(theme.muted_text)
                                                });
                                            });
                                    });

                                // Visibility icon (eye-on / eye-off)
                                let vis_icon = if layer.visible() {
                                    IconId::EyeOn
                                } else {
                                    IconId::EyeOff
                                };
                                let vis_tex = state.icon_cache.get(vis_icon);
                                ui.element()
                                    .width(fixed!(20.0))
                                    .height(fixed!(20.0))
                                    .background_color(theme.muted_text)
                                    .image(vis_tex)
                                    .empty();

                                // Lock icon
                                let lock_tex = state.icon_cache.get(IconId::Unlock);
                                ui.element()
                                    .width(fixed!(18.0))
                                    .height(fixed!(18.0))
                                    .background_color(theme.muted_text)
                                    .image(lock_tex)
                                    .empty();
                            });
                    }
                });
        });

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Layers);
}
