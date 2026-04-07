use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::embedded_samples::embedded_samples;
use crate::l10n;
use crate::session_ops::load_sample_by_path;
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, dashboard_nav_items};

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    // Header
    ui.element()
        .id("dash-header")
        .width(grow!())
        .height(fixed!(44.0))
        .background_color(theme.background_elevated)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((0, 16, 0, 16))
        })
        .children(|ui| {
            let _title = l10n::text(state.resolved_language(), "nav-projects");
            ui.text("Project Dashboard", |t| t.font_size(17).color(theme.text));
            ui.element().width(grow!()).height(fixed!(1.0)).empty();
            ui.text("+ New Project", |t| t.font_size(15).color(theme.muted_text));
        });

    // Create New Project button
    ui.element().width(grow!()).height(fixed!(16.0)).empty();

    ui.element()
        .id("create-new")
        .width(grow!(min: 0.0, max: 360.0))
        .height(fixed!(52.0))
        .background_color(theme.surface_elevated)
        .corner_radius(12.0)
        .layout(|l| l.align(CenterX, CenterY).padding((0, 16, 0, 16)))
        .children(|ui| {
            ui.text("+ Create New Project", |t| {
                t.font_size(16).color(theme.text).alignment(CenterX)
            });
        });

    ui.element().width(grow!()).height(fixed!(12.0)).empty();

    // Project cards
    ui.element()
        .id("project-list")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).gap(2).padding((0, 16, 0, 16)))
        .overflow(|o| {
            o.scroll_y().scrollbar(|s| {
                s.width(3.0)
                    .thumb_color(theme.border_strong)
                    .track_color(theme.surface)
                    .hide_after_frames(120)
            })
        })
        .children(|ui| {
            let samples = embedded_samples();
            for (i, sample) in samples.iter().enumerate() {
                let path = sample.path;
                ui.element()
                    .id(("project-card", i as u32))
                    .width(grow!())
                    .height(fixed!(88.0))
                    .background_color(theme.surface)
                    .corner_radius(10.0)
                    .layout(|l| {
                        l.direction(LeftToRight)
                            .align(Left, CenterY)
                            .gap(12)
                            .padding((12, 12, 12, 12))
                    })
                    .border(|b| b.bottom(1).color(theme.border))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            load_sample_by_path(state, path);
                            state.navigate(MobileScreen::Editor);
                        }

                        // Thumbnail
                        ui.element()
                            .id(("thumb", i as u32))
                            .width(fixed!(64.0))
                            .height(fixed!(64.0))
                            .corner_radius(8.0)
                            .image(sample.thumb)
                            .empty();

                        // Text info
                        ui.element()
                            .width(grow!())
                            .height(fit!())
                            .layout(|l| l.direction(TopToBottom).gap(4))
                            .children(|ui| {
                                ui.text(sample.title, |t| t.font_size(16).color(theme.text));
                                ui.text(sample.meta, |t| t.font_size(12).color(theme.muted_text));
                            });
                    });
            }
        });

    // Bottom nav
    let items = dashboard_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Dashboard);
}
