use ply_engine::prelude::*;

use crate::app_state::{AppState, MobileScreen};
use crate::embedded_samples::{embedded_sample, embedded_samples};
use crate::icons::IconId;
use crate::l10n;
use crate::session_ops::load_sample_by_path;
use crate::theme::PlyTheme;
use crate::workspace::{self, BUILTIN_WORKSPACE, MapFileInfo};

use super::widgets::{bottom_nav, dashboard_nav_items};

// ── Workspace-aware header (title + chevron-down) ───────────────────

fn workspace_header(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let title = if state.active_workspace == BUILTIN_WORKSPACE {
        l10n::text(lang, "dashboard-workspace-builtin")
    } else {
        state.active_workspace.clone()
    };

    ui.element()
        .id("dash-header")
        .width(grow!())
        .height(fixed!(56.0))
        .background_color(theme.background_elevated)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(CenterX, CenterY)
                .padding((20, 16, 16, 16))
                .gap(6)
        })
        .children(|ui| {
            // Left spacer
            ui.element().width(fixed!(92.0)).height(fixed!(1.0)).empty();

            // Center: workspace name + chevron (tappable to open picker)
            ui.element()
                .id("dash-ws-title")
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(CenterX, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.show_workspace_picker = !state.show_workspace_picker;
                    }
                    ui.element()
                        .width(fit!())
                        .height(fit!())
                        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY).gap(6))
                        .children(|ui| {
                            ui.text(&title, |t| t.font_size(17).color(theme.text));
                            let chevron_tex = state.icon_cache.get(IconId::ChevronDown);
                            ui.element()
                                .width(fixed!(14.0))
                                .height(fixed!(14.0))
                                .background_color(theme.muted_text)
                                .image(chevron_tex)
                                .empty();
                        });
                });

            // Right spacer
            ui.element().width(fixed!(92.0)).height(fixed!(1.0)).empty();
        });
}

// ── Workspace picker popup ──────────────────────────────────────────

#[expect(clippy::excessive_nesting)] // reason: Ply UI popup requires nested closures
fn workspace_picker(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    if !state.show_workspace_picker {
        return;
    }

    let lang = state.resolved_language();
    let sw = screen_width();
    let sh = screen_height();

    // Semi-transparent backdrop
    ui.element()
        .id("ws-picker-backdrop")
        .width(fixed!(sw))
        .height(fixed!(sh))
        .background_color(Color::u_rgba(0, 0, 0, 120))
        .floating(|f| f.attach_root().offset((0.0, 0.0)))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.show_workspace_picker = false;
            }
        });

    // Popup card, positioned below header
    let popup_w: f32 = 300.0;
    let popup_x = (sw - popup_w) / 2.0;
    ui.element()
        .id("ws-picker-popup")
        .width(fixed!(popup_w))
        .height(fit!())
        .background_color(theme.surface_elevated)
        .corner_radius(16.0)
        .border(|b| b.all(1).color(theme.border))
        .floating(|f| f.attach_root().offset((popup_x, 66.0)))
        .layout(|l| l.direction(TopToBottom).padding((8, 12, 8, 12)))
        .children(|ui| {
            let workspaces: Vec<String> = state.workspace_list.clone();
            for (i, ws_name) in workspaces.iter().enumerate() {
                let is_active = *ws_name == state.active_workspace;
                let display_name = if ws_name == BUILTIN_WORKSPACE {
                    l10n::text(lang, "dashboard-workspace-builtin")
                } else {
                    ws_name.clone()
                };
                let text_color = if is_active { theme.accent } else { theme.text };

                ui.element()
                    .id(("ws-item", i as u32))
                    .width(grow!())
                    .height(fixed!(44.0))
                    .layout(|l| l.align(Left, CenterY).padding((14, 0, 14, 0)))
                    .on_press(move |_, _| {})
                    .children(|ui| {
                        if ui.just_released() {
                            state.active_workspace = ws_name.clone();
                            state.show_workspace_picker = false;
                        }
                        ui.text(&display_name, |t| t.font_size(16).color(text_color));
                    });
            }
        });
}

// ── Action buttons row (New + Import) ───────────────────────────────

fn action_buttons(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let new_label = l10n::text(lang, "dashboard-action-new");
    let import_label = l10n::text(lang, "dashboard-action-import");
    let export_label = l10n::text(lang, "dashboard-action-export");

    ui.element()
        .width(grow!())
        .height(fixed!(56.0))
        .layout(|l| l.direction(LeftToRight).gap(10))
        .children(|ui| {
            // "New" button
            dash_action_btn(ui, state, theme, "dash-btn-new", IconId::Plus, &new_label, 0);
            // "Import" button
            dash_action_btn(ui, state, theme, "dash-btn-import", IconId::Import, &import_label, 1);
            // "Export" button
            dash_action_btn(ui, state, theme, "dash-btn-export", IconId::Export, &export_label, 2);
        });
}

fn dash_action_btn(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    id: &'static str,
    icon_id: IconId,
    label: &str,
    action: u8,
) {
    let icon_tex = state.icon_cache.get(icon_id);
    ui.element()
        .id(id)
        .width(grow!())
        .height(grow!())
        .background_color(theme.surface)
        .corner_radius(14.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY).gap(6))
        .on_press(move |_, _| {})
        .children(|ui| {
            handle_dash_action(ui, state, action);
            ui.element()
                .width(fixed!(16.0))
                .height(fixed!(16.0))
                .background_color(theme.text)
                .image(icon_tex)
                .empty();
            ui.text(label, |t| t.font_size(14).color(theme.text));
        });
}

fn handle_dash_action(ui: &mut Ui, state: &mut AppState, action: u8) {
    if !ui.just_released() {
        return;
    }
    let lang = state.resolved_language();
    match action {
        0 => {
            let name = next_workspace_name(&state.workspace_list);
            if workspace::create_workspace(&name).is_some() {
                state.active_workspace = name;
                state.workspace_list =
                    workspace::list_workspaces().into_iter().map(|w| w.name).collect();
                state.status = l10n::text(lang, "dashboard-workspace-created");
            }
        }
        1 => {
            state.show_import_menu = !state.show_import_menu;
        }
        2 => {
            if let Some(root) = workspace::workspaces_root() {
                let ws_path = root.join(&state.active_workspace);
                crate::platform::launch_export_zip(&ws_path.to_string_lossy());
            }
        }
        _ => {}
    }
}

// ── Project list (from workspace or embedded fallback) ──────────────

fn project_list_builtin(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
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
                        ui.text(sample.title, |t| t.font_size(16).color(theme.text));
                        ui.text(sample.subtitle, |t| t.font_size(13).color(theme.muted_text));
                        ui.text(sample.meta, |t| t.font_size(13).color(theme.muted_text));
                    });
            });
    }
}

#[expect(clippy::excessive_nesting)] // reason: Ply UI project rows with nested children
fn project_list_filesystem(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    maps: &[MapFileInfo],
) {
    let lang = state.resolved_language();
    if maps.is_empty() {
        ui.element()
            .width(grow!())
            .height(fixed!(80.0))
            .layout(|l| l.align(CenterX, CenterY))
            .children(|ui| {
                let msg = l10n::text(lang, "dashboard-workspace-empty");
                ui.text(&msg, |t| t.font_size(14).color(theme.muted_text));
            });
        return;
    }
    for (i, map_info) in maps.iter().enumerate() {
        let is_first = i == 0;
        let full_path = map_info.path.to_string_lossy().into_owned();
        let file_name = map_info.file_name.clone();
        let size_kb = map_info.size_bytes as f64 / 1024.0;
        let size_str = if size_kb < 1.0 {
            format!("{} B", map_info.size_bytes)
        } else {
            format!("{size_kb:.1} KB")
        };

        // Try to match to an embedded sample for the thumbnail
        let thumb = embedded_sample(&file_name).map(|s| s.thumb);

        ui.element()
            .id(("fs-row", i as u32))
            .width(grow!())
            .height(fixed!(80.0))
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
                    crate::session_ops::load_filesystem_map(state, &full_path);
                    state.navigate(MobileScreen::Editor);
                }

                // Thumbnail or placeholder
                if let Some(thumb_asset) = thumb {
                    ui.element()
                        .id(("fs-thumb", i as u32))
                        .width(fixed!(48.0))
                        .height(fixed!(48.0))
                        .corner_radius(10.0)
                        .image(thumb_asset)
                        .empty();
                } else {
                    ui.element()
                        .id(("fs-thumb", i as u32))
                        .width(fixed!(48.0))
                        .height(fixed!(48.0))
                        .corner_radius(10.0)
                        .background_color(theme.surface)
                        .border(|b| b.all(1).color(theme.border))
                        .layout(|l| l.align(CenterX, CenterY))
                        .children(|ui| {
                            let icon = state.icon_cache.get(IconId::NavProjects);
                            ui.element()
                                .width(fixed!(20.0))
                                .height(fixed!(20.0))
                                .background_color(theme.muted_text)
                                .image(icon)
                                .empty();
                        });
                }

                // Text block
                ui.element()
                    .width(grow!())
                    .height(fit!())
                    .layout(|l| l.direction(TopToBottom).gap(4))
                    .children(|ui| {
                        ui.text(&file_name, |t| t.font_size(16).color(theme.text));
                        ui.text(&size_str, |t| t.font_size(13).color(theme.muted_text));
                    });
            });
    }
}

// ── Main render ─────────────────────────────────────────────────────

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    workspace_header(ui, state, theme);

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
            action_buttons(ui, state, theme);

            ui.element().width(grow!()).height(fixed!(14.0)).empty();

            // Project list panel
            ui.element()
                .id("project-list-panel")
                .width(grow!())
                .height(fit!())
                .background_color(theme.surface_elevated)
                .corner_radius(20.0)
                .border(|b| b.all(1).color(theme.border))
                .layout(|l| l.direction(TopToBottom))
                .children(|ui| {
                    if state.active_workspace == BUILTIN_WORKSPACE {
                        project_list_builtin(ui, state, theme);
                    } else {
                        let maps = workspace::list_maps(&state.active_workspace);
                        project_list_filesystem(ui, state, theme, &maps);
                    }
                });
        });

    // Workspace picker overlay (rendered last, on top)
    workspace_picker(ui, state, theme);

    // Import submenu overlay
    super::dashboard_import::import_menu_popup(ui, state, theme);

    let items = dashboard_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Dashboard);
}

/// Generate a unique workspace name like "workspace-1", "workspace-2", etc.
fn next_workspace_name(existing: &[String]) -> String {
    let mut n = 1u32;
    loop {
        let name = format!("workspace-{n}");
        if !existing.iter().any(|e| e == &name) {
            return name;
        }
        n += 1;
    }
}
