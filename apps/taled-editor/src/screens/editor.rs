use ply_engine::prelude::*;

use crate::app_state::{
    AppState, MobileScreen, ShapeFillMode, TileSelectionMode, Tool, is_tile_selection_tool,
};
use crate::canvas::render_canvas;
use crate::l10n;
use crate::theme::PlyTheme;

use super::editor_toolbar::render_toolbar;
use super::tile_palette::render_viewfinder;
use super::widgets::{bottom_nav, editor_nav_items};

pub(crate) fn render(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    render_editor_header(ui, state, theme);
    render_tile_strip_shell(ui, state, theme);

    // Canvas fills remaining space between tile strip and toolbar
    render_canvas(ui, state, theme);

    render_toolbar(ui, state, theme);

    let items = editor_nav_items();
    bottom_nav(ui, state, theme, &items, MobileScreen::Editor);
}

fn render_editor_header(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let title = state
        .session
        .as_ref()
        .map(|s| {
            s.document()
                .file_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Untitled".to_string())
        })
        .unwrap_or_else(|| "Tile Map Editor".to_string());

    ui.element()
        .id("editor-header")
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
            // Left: Back button (92px)
            let back = l10n::text(state.resolved_language(), "common-back");
            ui.element()
                .id("editor-back")
                .width(fixed!(92.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.navigate_back_to(MobileScreen::Dashboard);
                    }
                    ui.text(&back, |t| {
                        t.font_size(14).color(super::widgets::HEADER_ACTION_COLOR)
                    });
                });

            // Center: title
            ui.element()
                .width(grow!())
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    ui.text(&title, |t| {
                        t.font_size(17).color(theme.text).alignment(CenterX)
                    });
                });

            // Right: Settings (92px)
            ui.element()
                .id("editor-settings")
                .width(fixed!(92.0))
                .height(grow!())
                .layout(|l| l.align(Right, CenterY))
                .on_press(move |_, _| {})
                .children(|ui| {
                    if ui.just_released() {
                        state.navigate(MobileScreen::Settings);
                    }
                    let settings = l10n::text(state.resolved_language(), "nav-settings");
                    ui.text(&settings, |t| {
                        t.font_size(14).color(theme.muted_text).alignment(Right)
                    });
                });
        });
}

/// Tile strip shell — 114px, sits between header and canvas.
/// Contains palette area (left) + side divider + tool panel (right).
/// When the active layer is an object layer, the palette area shows object info instead.
fn render_tile_strip_shell(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let strip_bg = theme.surface_elevated;
    let divider_color = Color::rgba(1.0, 1.0, 1.0, 0.10);
    let is_obj_layer = state.active_layer_is_object();

    ui.element()
        .id("tile-strip-shell")
        .width(grow!())
        .height(fixed!(114.0))
        .background_color(strip_bg)
        .border(|b| b.bottom(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight))
        .children(|ui| {
            // Left: tile viewfinder OR object info panel
            ui.element()
                .id("tile-palette")
                .width(grow!())
                .height(grow!())
                .overflow(|o| o.clip())
                .on_press(move |_, _| {})
                .children(|ui| {
                    if is_obj_layer {
                        render_object_info_panel(ui, state, theme);
                    } else {
                        render_viewfinder(ui, state, theme);
                    }
                });

            // Vertical divider
            ui.element()
                .width(fixed!(1.0))
                .height(grow!())
                .layout(|l| l.padding((10, 0, 10, 0)))
                .children(|ui| {
                    ui.element()
                        .width(fixed!(1.0))
                        .height(grow!())
                        .background_color(divider_color)
                        .corner_radius(0.5)
                        .empty();
                });

            // Right: tool side panel (62px)
            render_tool_side_panel(ui, state, theme);
        });
}

fn render_tool_side_panel(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();
    let selection_active = is_tile_selection_tool(state.tool);
    let shape_fill_active = state.tool == Tool::ShapeFill;
    let has_options = selection_active || shape_fill_active;

    if has_options {
        ui.element()
            .id("tool-side-panel")
            .width(fixed!(62.0))
            .height(grow!())
            .overflow(|o| o.scroll_y())
            .layout(|l| {
                l.direction(TopToBottom)
                    .padding((8, 4, 8, 4))
                    .gap(3)
                    .align(CenterX, Top)
            })
            .children(|ui| {
                if selection_active {
                    render_selection_modes(ui, state, theme, lang);
                } else {
                    render_shape_fill_modes(ui, state, theme, lang);
                }
            });
    } else {
        render_side_empty(ui, theme, lang);
    }
}

fn render_side_empty(ui: &mut Ui, theme: &PlyTheme, lang: l10n::SupportedLanguage) {
    let empty_color = Color::u_rgb(0x6e, 0x6e, 0x73);
    let _ = theme;
    let line1 = l10n::text(lang, "tile-strip-side-empty-line-1");
    let line2 = l10n::text(lang, "tile-strip-side-empty-line-2");
    let combined = format!("{line1}\n{line2}");
    // Strip is 114px; text block ≈ 26px; top padding 44 centres it vertically.
    ui.element()
        .id("tool-side-panel")
        .width(fixed!(62.0))
        .height(grow!())
        .layout(|l| l.direction(TopToBottom).padding((44, 0, 0, 0)))
        .children(|ui| {
            ui.text(&combined, |t| {
                t.font_size(9).color(empty_color).alignment(CenterX)
            });
        });
}

use crate::icons::IconId;

fn render_mode_button(
    ui: &mut Ui,
    state: &mut AppState,
    id: &'static str,
    label: &str,
    active: bool,
    icon_id: IconId,
) -> bool {
    let text_color = if active {
        Color::u_rgb(0xff, 0xff, 0xff)
    } else {
        Color::u_rgb(0xd1, 0xd1, 0xd6)
    };
    let bg = if active {
        Color::u_rgba(142, 142, 147, 46)
    } else {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    };
    let icon_tex = state.icon_cache.get(icon_id);
    let mut released = false;

    ui.element()
        .id(id)
        .width(grow!())
        .height(fixed!(52.0))
        .background_color(bg)
        .corner_radius(9.0)
        .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(2))
        .on_press(move |_, _| {})
        .children(|ui| {
            released = ui.just_released();
            ui.element()
                .width(fixed!(22.0))
                .height(fixed!(22.0))
                .background_color(text_color)
                .image(icon_tex)
                .empty();
            ui.text(label, |t| t.font_size(9).color(text_color));
        });
    released
}

fn render_selection_modes(
    ui: &mut Ui,
    state: &mut AppState,
    _theme: &PlyTheme,
    lang: l10n::SupportedLanguage,
) {
    let modes: [(TileSelectionMode, &str, IconId, &'static str); 4] = [
        (
            TileSelectionMode::Replace,
            "selection-mode-replace",
            IconId::ModeSelReplace,
            "sel-replace",
        ),
        (
            TileSelectionMode::Add,
            "selection-mode-add",
            IconId::ModeSelAdd,
            "sel-add",
        ),
        (
            TileSelectionMode::Subtract,
            "selection-mode-subtract",
            IconId::ModeSelSubtract,
            "sel-sub",
        ),
        (
            TileSelectionMode::Intersect,
            "selection-mode-intersect",
            IconId::ModeSelIntersect,
            "sel-inter",
        ),
    ];
    for (mode, key, icon_id, id) in &modes {
        let active = state.tile_selection_mode == *mode;
        let label = l10n::text(lang, key);
        let mode_val = *mode;
        if render_mode_button(ui, state, id, &label, active, *icon_id) {
            state.tile_selection_mode = mode_val;
        }
    }
}

fn render_shape_fill_modes(
    ui: &mut Ui,
    state: &mut AppState,
    _theme: &PlyTheme,
    lang: l10n::SupportedLanguage,
) {
    let modes: [(ShapeFillMode, &str, IconId, &'static str); 2] = [
        (
            ShapeFillMode::Rectangle,
            "shape-fill-mode-rectangle",
            IconId::ModeRectangle,
            "shp-rect",
        ),
        (
            ShapeFillMode::Ellipse,
            "shape-fill-mode-ellipse",
            IconId::ModeEllipse,
            "shp-ellip",
        ),
    ];
    for (mode, key, icon_id, id) in &modes {
        let active = state.shape_fill_mode == *mode;
        let label = l10n::text(lang, key);
        let mode_val = *mode;
        if render_mode_button(ui, state, id, &label, active, *icon_id) {
            state.shape_fill_mode = mode_val;
        }
    }
}

// Toolbar and floating controls extracted to editor_toolbar module.

/// Panel shown in place of the tile viewfinder when the active layer is an object layer.
/// Displays the selected object's name and editable position/size fields,
/// or a hint when nothing is selected.
fn render_object_info_panel(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();

    // Gather selected object info while session borrow is short.
    let obj_info: Option<(String, f32, f32, f32, f32)> = state.selected_object.and_then(|obj_id| {
        let session = state.session.as_ref()?;
        let layer = session.document().map.layer(state.active_layer)?;
        let obj_layer = layer.as_object()?;
        let obj = obj_layer.objects.iter().find(|o| o.id == obj_id)?;
        let label = if obj.name.is_empty() {
            format!("Object #{}", obj.id)
        } else {
            obj.name.clone()
        };
        Some((label, obj.x, obj.y, obj.width, obj.height))
    });

    // Sync text input values when selection changes (or on first frame).
    let selection_changed = state.obj_info_synced_for != state.selected_object;
    if selection_changed {
        if let Some((_, x, y, w, h)) = &obj_info {
            ui.set_text_value("obj-x", &format!("{x:.1}"));
            ui.set_text_value("obj-y", &format!("{y:.1}"));
            ui.set_text_value("obj-w", &format!("{w:.1}"));
            ui.set_text_value("obj-h", &format!("{h:.1}"));
        }
        state.obj_info_synced_for = state.selected_object;
    }

    ui.element()
        .id("obj-info-panel")
        .width(grow!())
        .height(grow!())
        .layout(|l| {
            l.direction(TopToBottom)
                .align(Left, CenterY)
                .padding((8, 12, 8, 12))
                .gap(4)
        })
        .children(|ui| {
            if let Some((name, _, _, _, _)) = obj_info {
                ui.text(&name, |t| t.font_size(14).color(theme.text));
                obj_field_row(ui, theme, "X", "obj-x", "Y", "obj-y");
                obj_field_row(ui, theme, "W", "obj-w", "H", "obj-h");
            } else {
                let hint = l10n::text(lang, "obj-info-no-selection");
                ui.text(&hint, |t| t.font_size(13).color(theme.muted_text));
            }
        });

    // After rendering, apply text values only when no obj field is focused (user finished editing).
    let editing = ui.focused_element().is_some_and(|f| {
        ["obj-x", "obj-y", "obj-w", "obj-h"]
            .iter()
            .any(|&id| f == Id::new(id))
    });
    if state.selected_object.is_some() && !editing {
        apply_obj_field(ui, state, "obj-x", ObjField::X);
        apply_obj_field(ui, state, "obj-y", ObjField::Y);
        apply_obj_field(ui, state, "obj-w", ObjField::W);
        apply_obj_field(ui, state, "obj-h", ObjField::H);
    }
}

/// Render a row with two label+input pairs: `A [___] B [___]`.
fn obj_field_row(
    ui: &mut Ui,
    theme: &PlyTheme,
    label_a: &str,
    id_a: &'static str,
    label_b: &str,
    id_b: &'static str,
) {
    let input_bg = Color::rgba(0.0, 0.0, 0.0, 0.25);
    ui.element()
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY).gap(4))
        .width(grow!())
        .children(|ui| {
            obj_field(ui, theme, label_a, id_a, input_bg);
            obj_field(ui, theme, label_b, id_b, input_bg);
        });
}

/// A single label + text input pair.
fn obj_field(ui: &mut Ui, theme: &PlyTheme, label: &str, field_id: &'static str, input_bg: Color) {
    ui.element().width(fixed!(14.0)).children(|ui| {
        ui.text(label, |t| t.font_size(12).color(theme.muted_text));
    });
    ui.element()
        .id(field_id)
        .width(grow!())
        .height(fixed!(26.0))
        .background_color(input_bg)
        .corner_radius(4.0)
        .layout(|l| l.padding((0, 4, 0, 4)).align(Left, CenterY))
        .text_input(|t| {
            t.font_size(12)
                .text_color(theme.text)
                .cursor_color(theme.text)
                .max_length(12)
        })
        .empty();
}

/// Which object field a text input corresponds to.
enum ObjField {
    X,
    Y,
    W,
    H,
}

/// Read a text input value and, if it parses as f32, apply it to the selected object.
fn apply_obj_field(ui: &Ui, state: &mut AppState, field_id: &'static str, field: ObjField) {
    let text = ui.get_text_value(field_id);
    if text.is_empty() {
        return;
    }
    let Ok(val) = text.parse::<f32>() else {
        return;
    };

    let Some(obj_id) = state.selected_object else {
        return;
    };
    let Some(session) = state.session.as_mut() else {
        return;
    };
    let doc = session.document_mut();
    let Some(layer) = doc.map.layer_mut(state.active_layer) else {
        return;
    };
    let Some(ol) = layer.as_object_mut() else {
        return;
    };
    let Some(obj) = ol.object_mut(obj_id) else {
        return;
    };

    let current = match field {
        ObjField::X => obj.x,
        ObjField::Y => obj.y,
        ObjField::W => obj.width,
        ObjField::H => obj.height,
    };

    // Only apply if the parsed value actually differs (avoids marking dirty every frame).
    if (val - current).abs() > 0.001 {
        match field {
            ObjField::X => obj.x = val,
            ObjField::Y => obj.y = val,
            ObjField::W => obj.width = val,
            ObjField::H => obj.height = val,
        }
        state.canvas_dirty = true;
    }
}
