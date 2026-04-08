use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::theme::PlyTheme;

use super::widgets::{bottom_nav, editor_nav_items, page_header};

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    page_header(
        ui,
        theme,
        "Object Library",
        Some(("Back", MobileScreen::Editor)),
        Some(("Done", MobileScreen::Objects)),
        state,
    );

    ui.element()
        .id("objects-body")
        .width(grow!())
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((12, 14, 8, 14)).gap(10))
        .overflow(|o| {
            o.scroll_y().scrollbar(|s| {
                s.width(3.0)
                    .thumb_color(theme.border_strong)
                    .track_color(theme.surface)
                    .hide_after_frames(120)
            })
        })
        .children(|ui| {
            // Search bar
            ui.element()
                .id("obj-search")
                .width(grow!())
                .height(fixed!(52.0))
                .background_color(theme.surface)
                .corner_radius(16.0)
                .border(|b| b.all(1).color(theme.border))
                .layout(|l| {
                    l.direction(LeftToRight)
                        .align(Left, CenterY)
                        .padding((0, 16, 0, 16))
                        .gap(10)
                })
                .children(|ui| {
                    ui.text("🔍", |t| t.font_size(14).color(theme.muted_text));
                    ui.text("Search objects...", |t| {
                        t.font_size(14).color(theme.muted_text)
                    });
                });

            // Object grid (3x3)
            let Some(session) = state.session.as_ref() else {
                ui.text("No map loaded", |t| t.font_size(14).color(theme.muted_text));
                return;
            };
            let map = &session.document().map;

            // Collect objects from object layers
            let mut objects: Vec<(String, String)> = Vec::new();
            for layer in &map.layers {
                if let Some(obj_layer) = layer.as_object() {
                    for obj in &obj_layer.objects {
                        let name = if obj.name.is_empty() {
                            format!("Object {}", obj.id)
                        } else {
                            obj.name.clone()
                        };
                        let kind = format!("{:?}", obj.shape);
                        objects.push((name, kind));
                    }
                }
            }

            // If no objects, add dummy placeholders for visual parity
            if objects.is_empty() {
                for i in 0..9 {
                    objects.push((
                        format!("Object {}", i + 1),
                        [
                            "NPC", "Trigger", "Spawn", "Chest", "Door", "Portal", "Sign", "Light",
                            "Sound",
                        ][i]
                            .to_string(),
                    ));
                }
            }

            // Grid layout: 3 columns
            let chunks: Vec<&[(String, String)]> = objects.chunks(3).collect();
            for (row_i, row) in chunks.iter().enumerate() {
                ui.element()
                    .id(("obj-row", row_i as u32))
                    .width(grow!())
                    .height(fixed!(150.0))
                    .layout(|l| l.direction(LeftToRight).gap(12))
                    .children(|ui| {
                        for (col_i, (name, kind)) in row.iter().enumerate() {
                            let idx = row_i * 3 + col_i;
                            ui.element()
                                .id(("obj-card", idx as u32))
                                .width(grow!())
                                .height(grow!())
                                .background_color(theme.surface)
                                .corner_radius(18.0)
                                .border(|b| b.all(1).color(theme.border))
                                .layout(|l| {
                                    l.direction(TopToBottom)
                                        .align(CenterX, CenterY)
                                        .gap(14)
                                        .padding((16, 10, 16, 10))
                                })
                                .children(|ui| {
                                    // Icon placeholder
                                    ui.element()
                                        .width(fixed!(58.0))
                                        .height(fixed!(58.0))
                                        .background_color(theme.surface_elevated)
                                        .corner_radius(16.0)
                                        .layout(|l| l.align(Left, CenterY))
                                        .children(|ui| {
                                            ui.text("◇", |t| {
                                                t.font_size(24)
                                                    .color(theme.accent)
                                                    .alignment(CenterX)
                                            });
                                        });
                                    ui.text(name, |t| {
                                        t.font_size(12).color(theme.text).alignment(CenterX)
                                    });
                                    ui.text(kind, |t| {
                                        t.font_size(10).color(theme.muted_text).alignment(CenterX)
                                    });
                                });
                        }
                        // Pad remaining columns
                        for _ in row.len()..3 {
                            ui.element().width(grow!()).height(grow!()).empty();
                        }
                    });
            }
        });

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Objects);
}
