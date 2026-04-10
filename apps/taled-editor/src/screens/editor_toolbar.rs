use ply_engine::prelude::*;

use crate::app_state::{AppState, Tool};
use crate::icons::IconId;
use crate::l10n;
use crate::session_ops::{adjust_zoom, apply_redo, apply_undo};
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
    let safe_top = state.safe_inset_top;
    // Top-anchored controls
    render_history_buttons(ui, state, theme, safe_top);
    render_layer_panel(ui, state, theme, safe_top);
    // Selection action bar (appears when selection or transfer is active)
    if state.tile_selection_cells.is_some() || state.tile_selection_transfer.is_some() {
        render_selection_actions(ui, state, theme, safe_top);
    }
    // Bottom-positioned controls (using Top anchor with calculated Y offsets)
    let canvas_h = (screen_height() - 56.0 - 114.0 - 68.0 - 72.0 - safe_top).max(200.0);
    render_dpad_float(ui, state, theme, canvas_h, safe_top);
    render_zoom_float(ui, state, theme, canvas_h, safe_top);
}

/// Floating bar with selection action buttons.
/// Shows Copy/Cut/Del when selection is active, Place/Cancel during transfer.
fn render_selection_actions(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, safe_top: f32) {
    let float_bg = Color::u_rgba(24, 24, 26, 245);
    let float_border = Color::u_rgba(255, 255, 255, 20);
    let has_transfer = state.tile_selection_transfer.is_some();
    ui.element()
        .id("sel-actions")
        .floating(|f| {
            f.anchor((Right, Top), (Right, Top))
                .attach_root()
                .offset((0.0, 218.0 + safe_top))
                .z_index(14)
        })
        .background_color(float_bg)
        .corner_radius(14.0)
        .border(|b| b.all(1).color(float_border))
        .layout(|l| l.direction(LeftToRight).padding((6, 8, 6, 8)).gap(4))
        .children(|ui| {
            if has_transfer {
                sel_action_button(ui, state, theme, "sel-place", "Place", SelAction::Place);
                sel_action_button(ui, state, theme, "sel-cancel", "✕", SelAction::Cancel);
            } else {
                sel_action_button(ui, state, theme, "sel-copy", "Copy", SelAction::Copy);
                sel_action_button(ui, state, theme, "sel-cut", "Cut", SelAction::Cut);
                sel_action_button(ui, state, theme, "sel-del", "Del", SelAction::Delete);
            }
        });
}

#[derive(Clone, Copy)]
enum SelAction {
    Copy,
    Cut,
    Delete,
    Place,
    Cancel,
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
        .width(fixed!(48.0))
        .height(fixed!(32.0))
        .background_color(theme.accent_soft)
        .corner_radius(8.0)
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                match action {
                    SelAction::Copy => crate::selection_ops::copy_tile_selection(state),
                    SelAction::Cut => crate::selection_ops::cut_tile_selection(state),
                    SelAction::Delete => crate::selection_ops::delete_selection(state),
                    SelAction::Place => crate::selection_ops::place_tile_selection_transfer(state),
                    SelAction::Cancel => {
                        crate::selection_ops::cancel_tile_selection_transfer(state)
                    }
                }
            }
            ui.text(label, |t| {
                t.font_size(12).color(theme.text).alignment(CenterX)
            });
        });
}

/// Bottom floating controls (D-pad + zoom) rendered at the editor/root level
/// using absolute Y positions since Bottom anchoring is unreliable in Ply.
fn render_dpad_float(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    canvas_h: f32,
    safe_top: f32,
) {
    // Position from screen top: safe_top + header(56) + tile_strip(114) + canvas_h - dpad(92) - margin(8)
    let dpad_y = safe_top + 56.0 + 114.0 + canvas_h - 92.0 - 8.0;
    ui.element()
        .id("dpad")
        .width(fixed!(92.0))
        .height(fixed!(92.0))
        .floating(|f| {
            f.anchor((Left, Top), (Left, Top))
                .attach_root()
                .offset((8.0, dpad_y))
                .z_index(10)
        })
        .background_color(theme.surface_elevated)
        .corner_radius(46.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.align(CenterX, CenterY))
        .children(|ui| {
            render_dpad_inner(ui, state, theme);
        });
}

fn render_zoom_float(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    canvas_h: f32,
    safe_top: f32,
) {
    let zoom_y = safe_top + 56.0 + 114.0 + canvas_h - 42.0 - 8.0;
    ui.element()
        .id("zoom-float")
        .width(fixed!(118.0))
        .height(fixed!(42.0))
        .floating(|f| {
            f.anchor((Right, Top), (Right, Top))
                .attach_root()
                .offset((-8.0, zoom_y))
                .z_index(10)
        })
        .background_color(theme.surface_elevated)
        .corner_radius(21.0)
        .border(|b| b.all(1).color(theme.border))
        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY))
        .children(|ui| {
            zoom_button(ui, state, theme, "zoom-out", "−", -25);
            ui.element()
                .width(fixed!(46.0))
                .height(grow!())
                .layout(|l| l.align(Left, CenterY))
                .children(|ui| {
                    let zoom_text = format!("{}%", state.zoom_percent);
                    ui.text(&zoom_text, |t| {
                        t.font_size(12).color(theme.muted_text).alignment(CenterX)
                    });
                });
            zoom_button(ui, state, theme, "zoom-in", "+", 25);
        });
}

fn render_history_buttons(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, safe_top: f32) {
    let session_can = state
        .session
        .as_ref()
        .map_or((false, false), |s| (s.can_undo(), s.can_redo()));
    let can_undo = !state.undo_action_order.is_empty() || session_can.0;
    let can_redo = !state.redo_action_order.is_empty() || session_can.1;

    let float_bg = Color::u_rgba(24, 24, 26, 245);
    let float_border = Color::u_rgba(255, 255, 255, 20);

    ui.element()
        .id("history-float")
        .floating(|f| {
            f.anchor((Left, Top), (Left, Top))
                .attach_root()
                .offset((6.0, 174.0 + safe_top))
                .z_index(12)
        })
        .layout(|l| l.direction(LeftToRight).gap(6))
        .children(|ui| {
            history_button(
                ui,
                state,
                theme,
                "undo",
                IconId::Undo,
                can_undo,
                float_bg,
                float_border,
                true,
            );
            history_button(
                ui,
                state,
                theme,
                "redo",
                IconId::Redo,
                can_redo,
                float_bg,
                float_border,
                false,
            );
        });
}

fn render_layer_panel(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme, safe_top: f32) {
    let lang = state.resolved_language();
    let layer_name = state
        .session
        .as_ref()
        .and_then(|s| s.document().map.layer(state.active_layer))
        .map_or_else(|| "\u{2014}".to_string(), |l| l.name().to_string());

    let float_bg = Color::u_rgba(24, 24, 26, 245);
    let float_border = Color::u_rgba(255, 255, 255, 20);
    let title_label = l10n::text(lang, "nav-layers");

    ui.element()
        .id("layer-float")
        .width(fixed!(158.0))
        .floating(|f| {
            f.anchor((Right, Top), (Right, Top))
                .attach_root()
                .offset((-6.0, 174.0 + safe_top))
                .z_index(12)
        })
        .background_color(float_bg)
        .corner_radius(14.0)
        .border(|b| b.all(1).color(float_border))
        .layout(|l| {
            l.direction(LeftToRight)
                .padding((8, 10, 6, 10))
                .align(Left, CenterY)
        })
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.layers_panel_expanded = !state.layers_panel_expanded;
            }
            ui.element()
                .width(grow!())
                .layout(|l| l.direction(TopToBottom).gap(1))
                .children(|ui| {
                    ui.text(&title_label, |t| t.font_size(12).color(theme.text));
                    ui.text(&layer_name, |t| {
                        t.font_size(10).color(Color::u_rgba(255, 255, 255, 168))
                    });
                });
            ui.text("▽", |t| t.font_size(14).color(theme.muted_text));
        });
}

fn render_dpad_inner(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    ui.element()
        .width(fixed!(60.0))
        .height(fixed!(60.0))
        .layout(|l| l.direction(TopToBottom).align(CenterX, CenterY).gap(4))
        .children(|ui| {
            dpad_button(ui, state, theme, "dpad-up", "▲", 0.0, -16.0);
            render_dpad_middle_row(ui, state, theme);
            dpad_button(ui, state, theme, "dpad-down", "▼", 0.0, 16.0);
        });
}

fn render_dpad_middle_row(ui: &mut Ui, state: &mut AppState, theme: &PlyTheme) {
    ui.element()
        .width(fixed!(60.0))
        .height(fixed!(18.0))
        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY).gap(6))
        .children(|ui| {
            dpad_button(ui, state, theme, "dpad-left", "◀", -16.0, 0.0);
            ui.element()
                .width(fixed!(16.0))
                .height(fixed!(16.0))
                .background_color(theme.surface)
                .corner_radius(8.0)
                .layout(|l| l.align(CenterX, CenterY))
                .children(|ui| {
                    ui.text("⊕", |t| {
                        t.font_size(11).color(theme.muted_text).alignment(CenterX)
                    });
                });
            dpad_button(ui, state, theme, "dpad-right", "▶", 16.0, 0.0);
        });
}

fn dpad_button(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    id: &'static str,
    glyph: &str,
    dx: f32,
    dy: f32,
) {
    let w = if dx != 0.0 { 14.0 } else { 24.0 };
    let h = if dx != 0.0 { 18.0 } else { 14.0 };
    ui.element()
        .id(id)
        .width(fixed!(w))
        .height(fixed!(h))
        .layout(|l| l.align(Left, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                state.pan_x += dx;
                state.pan_y += dy;
                state.canvas_dirty = true;
            }
            ui.text(glyph, |t| {
                t.font_size(12).color(theme.muted_text).alignment(CenterX)
            });
        });
}

fn zoom_button(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    id: &'static str,
    glyph: &str,
    delta: i32,
) {
    ui.element()
        .id(id)
        .width(fixed!(36.0))
        .height(grow!())
        .layout(|l| l.align(Left, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() {
                adjust_zoom(state, delta);
            }
            ui.text(glyph, |t| {
                t.font_size(18).color(theme.text).alignment(CenterX)
            });
        });
}

fn history_button(
    ui: &mut Ui,
    state: &mut AppState,
    _theme: &PlyTheme,
    id: &'static str,
    icon_id: IconId,
    enabled: bool,
    bg: Color,
    border_color: Color,
    is_undo: bool,
) {
    let icon_color = if enabled {
        Color::u_rgba(255, 255, 255, 235)
    } else {
        Color::u_rgba(255, 255, 255, 87)
    };
    let btn_bg = if enabled {
        bg
    } else {
        Color::u_rgba(28, 28, 30, 148)
    };
    let icon_tex = state.icon_cache.get(icon_id);

    ui.element()
        .id(id)
        .width(fixed!(38.0))
        .height(fixed!(38.0))
        .background_color(btn_bg)
        .corner_radius(19.0)
        .border(|b| b.all(1).color(border_color))
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_released() && enabled {
                if is_undo {
                    apply_undo(state);
                } else {
                    apply_redo(state);
                }
            }
            ui.element()
                .width(fixed!(20.0))
                .height(fixed!(20.0))
                .background_color(icon_color)
                .image(icon_tex)
                .empty();
        });
}
