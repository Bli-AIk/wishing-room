use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::embedded_samples::embedded_samples;
use crate::icons::IconId;
use crate::session_ops::load_sample_by_path;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, dashboard_nav_items, review_header};

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    review_header(ui, theme, "Project Dashboard", None, None);

    // Body: scrollable area
    ui.element()
        .id("dash-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((14, 14, 0, 14)))
        .overflow(|o| {
            o.scroll_y().scrollbar(|s| {
                s.width(3.0)
                    .thumb_color(theme.border_strong)
                    .track_color(theme.surface)
                    .hide_after_frames(120)
            })
        })
        .children(|ui| {
            // "Create New Project" button
            ui.element()
                .id("create-new")
                .width(grow!())
                .height(fixed!(68.0))
                .background_color(theme.surface)
                .corner_radius(16.0)
                .border(|b| b.all(1).color(theme.border))
                .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY).gap(10))
                .children(|ui| {
                    let plus_tex = state.icon_cache.get(IconId::Plus);
                    ui.element()
                        .width(fixed!(22.0))
                        .height(fixed!(22.0))
                        .background_color(theme.text)
                        .image(plus_tex)
                        .empty();
                    ui.text("Create New Project", |t| t.font_size(17).color(theme.text));
                });

            ui.element().width(grow!()).height(fixed!(18.0)).empty();

            // Project list panel (single rounded container)
            ui.element()
                .id("project-list-panel")
                .width(grow!())
                .height(fit!())
                .background_color(theme.surface_elevated)
                .corner_radius(20.0)
                .border(|b| b.all(1).color(theme.border))
                .layout(|l| l.direction(TopToBottom))
                .children(|ui| {
                    let samples = embedded_samples();
                    for (i, sample) in samples.iter().enumerate() {
                        let path = sample.path;
                        let is_first = i == 0;
                        ui.element()
                            .id(("project-row", i as u32))
                            .width(grow!())
                            .height(fixed!(104.0))
                            .layout(|l| {
                                l.direction(LeftToRight)
                                    .align(Left, CenterY)
                                    .gap(14)
                                    .padding((22, 14, 22, 14))
                            })
                            .border(|b| {
                                if is_first {
                                    b
                                } else {
                                    b.top(1).color(theme.border)
                                }
                            })
                            .on_press(move |_, _| {})
                            .children(|ui| {
                                if ui.just_released() {
                                    load_sample_by_path(state, path);
                                    state.navigate(MobileScreen::Editor);
                                }

                                // Thumbnail (60x60, 12px radius)
                                ui.element()
                                    .id(("thumb", i as u32))
                                    .width(fixed!(60.0))
                                    .height(fixed!(60.0))
                                    .corner_radius(12.0)
                                    .image(sample.thumb)
                                    .empty();

                                // Text block
                                ui.element()
                                    .width(grow!())
                                    .height(fit!())
                                    .layout(|l| l.direction(TopToBottom).gap(4))
                                    .children(|ui| {
                                        ui.text(sample.title, |t| {
                                            t.font_size(16).color(theme.text)
                                        });
                                        ui.text(sample.subtitle, |t| {
                                            t.font_size(13).color(theme.muted_text)
                                        });
                                        ui.text(sample.meta, |t| {
                                            t.font_size(13).color(theme.muted_text)
                                        });
                                    });
                            });
                    }
                });
        });

    let items = dashboard_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Dashboard);
}
