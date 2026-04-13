use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::icons::IconId;
use crate::l10n;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items, page_header};

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
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

    // Layer list
    ui.element()
        .id("layers-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((12, 14, 8, 14)).gap(16))
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

            for (i, display, is_obj, vis, locked) in layers.iter().rev() {
                let i = *i;
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
                            .padding((14, 14, 14, 14))
                            .gap(14)
                    })
                    .children(|ui| {
                        // Drag handle
                        ui.text("≡", |t| t.font_size(20).color(theme.muted_text));

                        // Layer thumbnail
                        ui.element()
                            .id(("layer-thumb", i as u32))
                            .width(fixed!(32.0))
                            .height(fixed!(32.0))
                            .background_color(theme.surface_elevated)
                            .corner_radius(6.0)
                            .border(|b| b.all(1).color(theme.border))
                            .empty();

                        // Layer info — tap to switch active layer
                        let kind = if *is_obj {
                            "Object Layer"
                        } else {
                            "Tile Layer"
                        };
                        ui.element()
                            .id(("layer-info", i as u32))
                            .width(grow!())
                            .height(fit!())
                            .layout(|l| l.direction(TopToBottom).gap(4))
                            .on_press(move |_, _| {})
                            .children(|ui| {
                                if ui.just_released() {
                                    state.active_layer = i;
                                }
                                ui.text(display, |t| t.font_size(15).color(theme.text));
                                ui.text(kind, |t| t.font_size(13).color(theme.muted_text));
                                opacity_bar(ui, theme);
                            });

                        // Eye icon (accent when visible)
                        let eye_id = if *vis { IconId::EyeOn } else { IconId::EyeOff };
                        let eye_c = if *vis { theme.accent } else { theme.muted_text };
                        let eye_tex = state.icon_cache.get(eye_id);
                        ui.element()
                            .width(fixed!(20.0))
                            .height(fixed!(20.0))
                            .background_color(eye_c)
                            .image(eye_tex)
                            .on_press(move |_, _| {})
                            .children(|ui| {
                                if ui.just_released() {
                                    let now_hidden =
                                        if state.hidden_layers.contains(&i) {
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

                        // Lock icon (accent when locked)
                        let lk_id = if *locked { IconId::Lock } else { IconId::Unlock };
                        let lk_c = if *locked {
                            theme.accent
                        } else {
                            theme.muted_text
                        };
                        let lk_tex = state.icon_cache.get(lk_id);
                        ui.element()
                            .width(fixed!(18.0))
                            .height(fixed!(18.0))
                            .background_color(lk_c)
                            .image(lk_tex)
                            .empty();
                    });
            }
        });

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Layers);
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
