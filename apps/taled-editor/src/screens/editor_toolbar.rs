use ply_engine::prelude::*;

use crate::app_state::{AppState, Tool};
use crate::l10n;
use crate::screens::editor_controls::alpha_scale;
use crate::theme::PlyTheme;

pub(crate) fn render_toolbar(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    let lang = state.resolved_language();

    // Dioxus tile toolbar order: Hand (pinned) | Paint, TerrainBrush*, Fill,
    // ShapeFill, Eraser, RectSelect, MagicWand, SameTile
    // * = placeholder (unimplemented)
    let toolbar_bg = theme.surface;

    ui.element()
        .id("toolbar")
        .width(grow!())
        .height(fixed!(68.0))
        .background_color(toolbar_bg)
        .border(|b| b.top(1).color(theme.border))
        .layout(|l| {
            l.direction(LeftToRight)
                .align(Left, CenterY)
                .padding((4, 8, 2, 8))
        })
        .children(|ui| {
            // Pinned Hand tool (always visible, not in scroll area)
            render_tool_item(ui, state, theme, lang, Tool::Hand, "tool-hand", 0);

            // Divider between pinned hand and scrollable tools
            ui.element()
                .width(fixed!(1.0))
                .height(fixed!(40.0))
                .background_color(theme.border)
                .empty();

            // Scrollable tool row
            ui.element()
                .id("tool-scroll-row")
                .width(grow!())
                .height(grow!())
                .overflow(|o| o.scroll_x())
                .layout(|l| {
                    l.direction(LeftToRight)
                        .align(Left, CenterY)
                        .padding((0, 6, 0, 6))
                        .gap(3)
                })
                .children(|ui| {
                    render_tool_item(ui, state, theme, lang, Tool::Paint, "tool-stamp", 1);
                    render_placeholder_tool(ui, state, lang, "tool-terrain-brush", 2);
                    render_tool_item(ui, state, theme, lang, Tool::Fill, "tool-fill", 3);
                    render_tool_item(
                        ui,
                        state,
                        theme,
                        lang,
                        Tool::ShapeFill,
                        "tool-shape-fill",
                        4,
                    );
                    render_tool_item(ui, state, theme, lang, Tool::Erase, "tool-eraser", 5);
                    render_tool_item(ui, state, theme, lang, Tool::Select, "tool-rect-select", 6);
                    render_tool_item(
                        ui,
                        state,
                        theme,
                        lang,
                        Tool::MagicWand,
                        "tool-magic-wand",
                        7,
                    );
                    render_tool_item(
                        ui,
                        state,
                        theme,
                        lang,
                        Tool::SelectSameTile,
                        "tool-same-tile",
                        8,
                    );
                });
        });
}

/// Renders a real (functional) tool button.
fn render_tool_item(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    lang: l10n::SupportedLanguage,
    tool: Tool,
    label_key: &str,
    index: u32,
) {
    let is_active = state.tool == tool;
    let label = l10n::text(lang, label_key);
    // Dioxus colors: inactive #8e8e93, active #d1d1d6
    let color = if is_active {
        Color::u_rgb(0xd1, 0xd1, 0xd6)
    } else {
        Color::u_rgb(0x8e, 0x8e, 0x93)
    };
    let bg = if is_active {
        theme.accent_soft
    } else {
        Color::rgba(0.0, 0.0, 0.0, 0.0)
    };

    let icon_id = crate::icons::tool_icon_id(label_key);
    let icon_tex = state.icon_cache.get(icon_id);

    ui.element()
        .id(("tool", index))
        .width(fixed!(60.0))
        .height(grow!())
        .background_color(bg)
        .corner_radius(10.0)
        .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(3))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.tool = tool;
            }
            ui.element()
                .width(fixed!(20.0))
                .height(fixed!(20.0))
                .background_color(color)
                .image(icon_tex)
                .empty();
            ui.text(&label, |t| t.font_size(10).color(color));
        });
}

/// Renders a placeholder (unimplemented) tool button — grayed out.
fn render_placeholder_tool(
    ui: &mut Ui,
    state: &mut AppState,
    lang: l10n::SupportedLanguage,
    label_key: &str,
    index: u32,
) {
    let label = l10n::text(lang, label_key);
    let placeholder_color = Color::u_rgb(0x6e, 0x6e, 0x73);

    let icon_id = crate::icons::tool_icon_id(label_key);
    let icon_tex = state.icon_cache.get(icon_id);

    ui.element()
        .id(("tool", index))
        .width(fixed!(60.0))
        .height(grow!())
        .corner_radius(10.0)
        .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(3))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                let status = l10n::text_with_args(
                    lang,
                    "tool-status-not-implemented",
                    &[("tool", label.clone())],
                );
                state.status = status;
            }
            ui.element()
                .width(fixed!(20.0))
                .height(fixed!(20.0))
                .background_color(placeholder_color)
                .image(icon_tex)
                .empty();
            ui.text(&label, |t| t.font_size(10).color(placeholder_color));
        });
}

pub(crate) fn render_floating_controls(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    if state.float_controls_alpha <= 0.0 {
        return;
    }
    let safe_top = state.safe_inset_top;
    // Top-anchored controls
    crate::screens::editor_controls::render_history_buttons(ui, state, theme, safe_top);
    crate::screens::editor_controls::render_layer_panel(ui, state, theme, safe_top);
    // Bottom-positioned controls (using Top anchor with calculated Y offsets)
    let canvas_h = (screen_height() - 56.0 - 114.0 - 68.0 - 72.0 - safe_top).max(200.0);
    let has_sel_actions =
        state.tile_selection_cells.is_some() || state.tile_selection_transfer.is_some();
    crate::screens::editor_controls::render_joystick_float(ui, state, theme, canvas_h, safe_top);
    // When selection actions are visible, push zoom up to make room.
    let zoom_extra_offset = if has_sel_actions { 52.0 } else { 0.0 };
    crate::screens::editor_controls::render_zoom_slider(
        ui,
        state,
        theme,
        canvas_h,
        safe_top,
        zoom_extra_offset,
    );
    // Selection action bar at bottom-right (below zoom)
    if has_sel_actions {
        render_selection_actions(ui, state, theme, canvas_h, safe_top);
    }
}

/// Floating bar with selection action buttons.
/// Normal: Cut / Copy / Flip X / Flip Y / Rotate / Delete
/// Transfer: Copy / Flip X / Flip Y / Rotate / Delete / Done
fn render_selection_actions(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    canvas_h: f32,
    safe_top: f32,
) {
    let a = state.float_controls_alpha;
    let float_bg = Color::u_rgba(24, 24, 26, alpha_scale(245, a));
    let float_border = Color::u_rgba(255, 255, 255, alpha_scale(20, a));
    let has_transfer = state.tile_selection_transfer.is_some();
    let sel_y = safe_top + 56.0 + 114.0 + canvas_h - 44.0 - 8.0;
    ui.element()
        .id("sel-actions")
        .height(fixed!(44.0))
        .floating(|f| {
            f.anchor((Right, Top), (Right, Top))
                .attach_root()
                .offset((-8.0, sel_y))
                .z_index(14)
        })
        .background_color(float_bg)
        .corner_radius(14.0)
        .border(|b| b.all(1).color(float_border))
        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY).padding((6, 6, 6, 6)).gap(2))
        .children(|ui| {
            if !has_transfer {
                sel_action_button(ui, state, theme, "sel-cut", "Cut", SelAction::Cut);
            }
            sel_action_button(ui, state, theme, "sel-copy", "Copy", SelAction::Copy);
            sel_action_button(ui, state, theme, "sel-fx", "Flip X", SelAction::FlipX);
            sel_action_button(ui, state, theme, "sel-fy", "Flip Y", SelAction::FlipY);
            sel_action_button(ui, state, theme, "sel-rot", "Rotate", SelAction::Rotate);
            sel_action_button(ui, state, theme, "sel-del", "Del", SelAction::Delete);
            if has_transfer {
                sel_action_button(ui, state, theme, "sel-done", "Done", SelAction::Done);
            }
        });
}

#[derive(Clone, Copy)]
enum SelAction {
    Copy,
    Cut,
    Delete,
    FlipX,
    FlipY,
    Rotate,
    Done,
}

fn sel_action_button(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    id: &'static str,
    label: &str,
    action: SelAction,
) {
    ui.element()
        .id(id)
        .width(fixed!(44.0))
        .height(fixed!(32.0))
        .background_color(theme.accent_soft)
        .corner_radius(8.0)
        .layout(|l| l.align(Left, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                match action {
                    SelAction::Copy => crate::selection_ops::copy_tile_selection(state),
                    SelAction::Cut => crate::selection_ops::cut_tile_selection(state),
                    SelAction::Delete => crate::selection_ops::delete_selection(state),
                    SelAction::FlipX => crate::selection_transform::flip_tile_selection_x(state),
                    SelAction::FlipY => crate::selection_transform::flip_tile_selection_y(state),
                    SelAction::Rotate => {
                        crate::selection_transform::rotate_tile_selection_cw(state)
                    }
                    SelAction::Done => {
                        crate::selection_ops::place_tile_selection_transfer(state)
                    }
                }
            }
            ui.text(label, |t| {
                t.font_size(11).color(theme.text).alignment(CenterX)
            });
        });
}
