use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::l10n;
use crate::theme::PlyTheme;

pub(crate) fn render_object_info_panel(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
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
                obj_name_row(ui, state, theme, &name);
                obj_field_row(ui, theme, "X", "obj-x", "Y", "obj-y");
                obj_field_row(ui, theme, "W", "obj-w", "H", "obj-h");
            } else {
                let hint = l10n::text(lang, "obj-info-no-selection");
                ui.text(&hint, |t| t.font_size(13).color(theme.muted_text));
            }
        });

    // After rendering, apply text values only when no obj field is focused.
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

/// Name row: object label, snap toggles, and delete button.
fn obj_name_row(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, name: &str) {
    ui.element()
        .layout(|l| l.direction(LeftToRight).align(Left, CenterY).gap(4))
        .width(grow!())
        .children(|ui| {
            ui.element().width(grow!()).children(|ui| {
                ui.text(name, |t| t.font_size(14).color(theme.text));
            });
            snap_icon_btn(
                ui,
                state,
                theme,
                "snap-grid",
                crate::icons::IconId::SnapGrid,
                true,
            );
            snap_icon_btn(
                ui,
                state,
                theme,
                "snap-int",
                crate::icons::IconId::SnapInt,
                false,
            );
            obj_delete_button(ui, state, theme);
        });
}

/// Icon toggle for snap-to-grid or snap-to-integer.
fn snap_icon_btn(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    id: &'static str,
    icon_id: crate::icons::IconId,
    is_grid: bool,
) {
    let active = if is_grid {
        state.snap_to_grid
    } else {
        state.snap_to_int
    };
    let bg = if active {
        theme.accent
    } else {
        Color::rgba(0.0, 0.0, 0.0, 0.25)
    };
    let fg = if active {
        Color::u_rgb(0xff, 0xff, 0xff)
    } else {
        theme.muted_text
    };
    let icon_tex = state.icon_cache.get(icon_id);
    ui.element()
        .id(id)
        .width(fixed!(28.0))
        .height(fixed!(24.0))
        .background_color(bg)
        .corner_radius(6.0)
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                toggle_snap(state, is_grid);
            }
            ui.element()
                .width(fixed!(16.0))
                .height(fixed!(16.0))
                .background_color(fg)
                .image(icon_tex)
                .empty();
        });
}

fn toggle_snap(state: &mut AppState, is_grid: bool) {
    if is_grid {
        state.snap_to_grid = !state.snap_to_grid;
        if state.snap_to_grid {
            state.snap_to_int = false;
        }
    } else {
        state.snap_to_int = !state.snap_to_int;
        if state.snap_to_int {
            state.snap_to_grid = false;
        }
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

/// Trash-icon button for deleting the selected object.
fn obj_delete_button(ui: &mut Ui, state: &mut AppState, _theme: &PlyTheme) {
    let del_bg = Color::u_rgba(0xff, 0x3b, 0x30, 180);
    let icon_tex = state.icon_cache.get(crate::icons::IconId::Trash);
    ui.element()
        .id("obj-delete-btn")
        .width(fixed!(28.0))
        .height(fixed!(24.0))
        .background_color(del_bg)
        .corner_radius(6.0)
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                delete_selected_object(state);
            }
            ui.element()
                .width(fixed!(16.0))
                .height(fixed!(16.0))
                .background_color(Color::u_rgb(0xff, 0xff, 0xff))
                .image(icon_tex)
                .empty();
        });
}

/// Delete the currently selected object (with undo support).
fn delete_selected_object(state: &mut AppState) {
    let Some(obj_id) = state.selected_object else {
        return;
    };
    let layer_idx = state.active_layer;
    let Some(session) = state.session.as_mut() else {
        return;
    };
    let result = session.edit(move |doc| {
        let layer = doc
            .map
            .layer_mut(layer_idx)
            .ok_or_else(|| taled_core::EditorError::Invalid("no layer".into()))?;
        let ol = layer
            .as_object_mut()
            .ok_or_else(|| taled_core::EditorError::Invalid("not object layer".into()))?;
        ol.remove_object(obj_id)
            .ok_or_else(|| taled_core::EditorError::Invalid("object not found".into()))?;
        Ok(())
    });
    match result {
        Ok(()) => {
            state.selected_object = None;
            state
                .undo_action_order
                .push(crate::app_state::UndoActionKind::DocumentEdit);
            state.redo_action_order.clear();
            state.canvas_dirty = true;
            state.tiles_dirty = true;
        }
        Err(e) => state.status = format!("Delete failed: {e}"),
    }
}
