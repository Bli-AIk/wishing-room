use ply_engine::prelude::*;

use crate::app_state::AppState;
use crate::icons::IconId;
use crate::l10n;
use crate::session_ops::{adjust_zoom, apply_redo, apply_undo};
use crate::theme::PlyTheme;

pub(crate) fn alpha_scale(base: u8, alpha: f32) -> u8 {
    ((base as f32) * alpha) as u8
}

pub(crate) fn render_history_buttons(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    safe_top: f32,
) {
    let session_can = state
        .session
        .as_ref()
        .map_or((false, false), |s| (s.can_undo(), s.can_redo()));
    let can_undo = !state.undo_action_order.is_empty() || session_can.0;
    let can_redo = !state.redo_action_order.is_empty() || session_can.1;

    let a = state.float_controls_alpha;
    let float_bg = Color::u_rgba(24, 24, 26, alpha_scale(245, a));
    let float_border = Color::u_rgba(255, 255, 255, alpha_scale(20, a));

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

#[expect(clippy::excessive_nesting)] // reason: Ply UI requires nested closures for element builders
pub(crate) fn render_layer_panel(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    safe_top: f32,
) {
    let lang = state.resolved_language();
    let layer_name = state
        .session
        .as_ref()
        .and_then(|s| s.document().map.layer(state.active_layer))
        .map_or_else(|| "\u{2014}".to_string(), |l| l.name().to_string());

    let a = state.float_controls_alpha;
    let float_bg = Color::u_rgba(24, 24, 26, alpha_scale(245, a));
    let float_border = Color::u_rgba(255, 255, 255, alpha_scale(20, a));
    let title_label = l10n::text(lang, "nav-layers");
    let expanded = state.layers_panel_expanded;
    let arrow = if expanded { "△" } else { "▽" };
    let panel_w = if expanded { 200.0 } else { 158.0 };

    ui.element()
        .id("layer-float")
        .width(fixed!(panel_w))
        .floating(|f| {
            f.anchor((Right, Top), (Right, Top))
                .attach_root()
                .offset((-6.0, 174.0 + safe_top))
                .z_index(12)
        })
        .background_color(float_bg)
        .corner_radius(14.0)
        .border(|b| b.all(1).color(float_border))
        .layout(|l| l.direction(TopToBottom).padding((0, 0, 0, 0)))
        .on_press(move |_, _| {})
        .children(|ui| {
            // Header — tap to toggle expand/collapse
            ui.element()
                .id("layer-float-header")
                .width(grow!())
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
                    ui.text(arrow, |t| t.font_size(14).color(theme.muted_text));
                });

            // Expanded layer list
            if expanded {
                let layers: Vec<(usize, String, bool, bool, bool)> = state
                    .session
                    .as_ref()
                    .map(|s| {
                        s.document()
                            .map
                            .layers
                            .iter()
                            .enumerate()
                            .map(|(i, l)| {
                                let is_obj = l.as_object().is_some();
                                let vis = l.visible() && !state.hidden_layers.contains(&i);
                                let locked = l.locked();
                                (i, l.name().to_string(), is_obj, vis, locked)
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                ui.element()
                    .id("layer-float-list")
                    .width(grow!())
                    .layout(|l| l.direction(TopToBottom).padding((2, 6, 8, 6)).gap(2))
                    .children(|ui| {
                        for (idx, name, is_obj, vis, locked) in layers.iter().rev() {
                            let idx = *idx;
                            let is_active = idx == state.active_layer;
                            let row_bg = if is_active {
                                theme.accent_soft
                            } else {
                                Color::u_rgba(0, 0, 0, 0)
                            };
                            let display = if name.is_empty() {
                                format!("Layer {idx}")
                            } else {
                                name.clone()
                            };

                            ui.element()
                                .id(("lf-row", idx as u32))
                                .width(grow!())
                                .height(fixed!(32.0))
                                .background_color(row_bg)
                                .corner_radius(8.0)
                                .layout(|l| {
                                    l.direction(LeftToRight)
                                        .align(Left, CenterY)
                                        .padding((4, 6, 4, 6))
                                        .gap(6)
                                })
                                .children(|ui| {
                                    // Eye icon — clickable
                                    let eye_id = if *vis {
                                        IconId::EyeOn
                                    } else {
                                        IconId::EyeOff
                                    };
                                    let eye_c = if *vis {
                                        theme.accent
                                    } else {
                                        theme.muted_text
                                    };
                                    let eye_tex = state.icon_cache.get(eye_id);
                                    ui.element()
                                        .id(("lf-eye", idx as u32))
                                        .width(fixed!(18.0))
                                        .height(fixed!(18.0))
                                        .background_color(eye_c)
                                        .image(eye_tex)
                                        .on_press(move |_, _| {})
                                        .children(|ui| {
                                            if ui.just_released() {
                                                let now_hidden =
                                                    if state.hidden_layers.contains(&idx) {
                                                        state.hidden_layers.remove(&idx);
                                                        false
                                                    } else {
                                                        state.hidden_layers.insert(idx);
                                                        true
                                                    };
                                                state.last_eye_toggle = Some((idx, now_hidden));
                                                state.tiles_dirty = true;
                                                state.canvas_dirty = true;
                                            }
                                        });

                                    // Name area — tap to switch active layer
                                    ui.element()
                                        .id(("lf-name", idx as u32))
                                        .width(grow!())
                                        .height(grow!())
                                        .layout(|l| {
                                            l.direction(LeftToRight)
                                                .align(Left, CenterY)
                                                .gap(6)
                                        })
                                        .on_press(move |_, _| {})
                                        .children(|ui| {
                                            if ui.just_released() {
                                                state.active_layer = idx;
                                                state.canvas_dirty = true;
                                            }
                                            let type_char = if *is_obj { "⊙" } else { "⊞" };
                                            ui.text(type_char, |t| {
                                                t.font_size(14).color(theme.muted_text)
                                            });
                                            ui.text(&display, |t| {
                                                t.font_size(13).color(theme.text)
                                            });
                                        });

                                    // Lock icon
                                    let lk_id = if *locked {
                                        IconId::Lock
                                    } else {
                                        IconId::Unlock
                                    };
                                    let lk_c = if *locked {
                                        theme.accent
                                    } else {
                                        theme.muted_text
                                    };
                                    let lk_tex = state.icon_cache.get(lk_id);
                                    ui.element()
                                        .width(fixed!(14.0))
                                        .height(fixed!(14.0))
                                        .background_color(lk_c)
                                        .image(lk_tex)
                                        .empty();
                                });
                        }
                    });
            }
        });
}

pub(crate) fn render_joystick_float(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    canvas_h: f32,
    safe_top: f32,
) {
    let outer = 108.0_f32;
    let mid = 72.0_f32;
    let knob_sz = 34.0_f32;
    let max_r = 18.0_f32;
    let joy_y = safe_top + 56.0 + 114.0 + canvas_h - outer - 8.0;
    let cx = 8.0 + outer / 2.0;
    let cy = joy_y + outer / 2.0;
    let a = state.float_controls_alpha;
    let ring_bg = Color::u_rgba(30, 30, 32, alpha_scale(255, a));
    let ring_border = Color::u_rgba(255, 255, 255, alpha_scale(12, a));
    let knob_color = Color::u_rgba(72, 72, 77, alpha_scale(255, a));

    ui.element()
        .id("joystick-outer")
        .width(fixed!(outer))
        .height(fixed!(outer))
        .floating(|f| {
            f.anchor((Left, Top), (Left, Top))
                .attach_root()
                .offset((8.0, joy_y))
                .z_index(10)
        })
        .background_color(ring_bg)
        .corner_radius(outer / 2.0)
        .border(|b| b.all(1).color(ring_border))
        .layout(|l| l.align(CenterX, CenterY))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_pressed() {
                state.joystick_active = true;
            }
            if ui.just_released() {
                state.joystick_active = false;
                state.joystick_offset = (0.0, 0.0);
            }
            // Middle ring — visual only
            ui.element()
                .id("joystick-mid")
                .width(fixed!(mid))
                .height(fixed!(mid))
                .background_color(theme.surface_elevated)
                .corner_radius(mid / 2.0)
                .border(|b| b.all(1).color(ring_border))
                .empty();
            // Knob — activation + visual
            joystick_knob(ui, state, knob_color, knob_sz);
            // Touch tracking — active flag set by outer ring or knob
            if state.joystick_active {
                let (mx, my) = mouse_position();
                let dx = mx - cx;
                let dy = my - cy;
                let dist = (dx * dx + dy * dy).sqrt().max(0.001);
                let (ox, oy) = if dist > max_r {
                    (dx * max_r / dist, dy * max_r / dist)
                } else {
                    (dx, dy)
                };
                state.joystick_offset = (ox, oy);
                let pan_speed = 3.0;
                state.pan_x -= ox * pan_speed / max_r;
                state.pan_y -= oy * pan_speed / max_r;
                state.canvas_dirty = true;
            }
        });
}

fn joystick_knob(ui: &mut Ui, state: &mut AppState, color: Color, sz: f32) {
    let border = Color::u_rgba(255, 255, 255, 25);
    let (kx, ky) = state.joystick_offset;
    ui.element()
        .id("joy-knob")
        .width(fixed!(sz))
        .height(fixed!(sz))
        .floating(|f| {
            f.anchor((CenterX, CenterY), (CenterX, CenterY))
                .attach_parent()
                .offset((kx, ky))
                .z_index(11)
        })
        .background_color(color)
        .corner_radius(sz / 2.0)
        .border(|b| b.all(1).color(border))
        .on_press(move |_, _| {})
        .layout(|l| l.align(CenterX, CenterY).padding((6, 0, 0, 0)))
        .children(|ui| {
            if ui.just_pressed() {
                state.joystick_active = true;
            }
            if ui.just_released() {
                state.joystick_active = false;
                state.joystick_offset = (0.0, 0.0);
            }
            ui.text("⊕", |t| {
                t.font_size(18).color(Color::u_rgba(200, 200, 205, 255))
            });
        });
}

pub(crate) fn render_zoom_slider(
    ui: &mut Ui,
    state: &mut AppState,
    theme: &PlyTheme,
    canvas_h: f32,
    safe_top: f32,
    extra_up: f32,
) {
    let outer_w = 158.0_f32;
    let outer_h = 46.0_f32;
    let inner_w = 96.0_f32;
    let inner_h = 32.0_f32;
    let handle_w = 62.0_f32;
    let handle_h = 26.0_f32;
    let max_offset = (inner_w - handle_w) / 2.0 - 2.0;
    let a = state.float_controls_alpha;
    let ring_bg = Color::u_rgba(30, 30, 32, alpha_scale(255, a));
    let ring_border = Color::u_rgba(255, 255, 255, alpha_scale(12, a));
    let handle_color = Color::u_rgba(72, 72, 77, alpha_scale(255, a));
    let zoom_y = safe_top + 56.0 + 114.0 + canvas_h - outer_h - 8.0 - extra_up;
    let slider_cx = screen_width() - 8.0 - outer_w / 2.0;

    ui.element()
        .id("zoom-slider")
        .width(fixed!(outer_w))
        .height(fixed!(outer_h))
        .floating(|f| {
            f.anchor((Right, Top), (Right, Top))
                .attach_root()
                .offset((-8.0, zoom_y))
                .z_index(10)
        })
        .background_color(ring_bg)
        .corner_radius(outer_h / 2.0)
        .border(|b| b.all(1).color(ring_border))
        .layout(|l| l.direction(LeftToRight).align(CenterX, CenterY).gap(8))
        .on_press(move |_, _| {})
        .children(|ui| {
            if ui.just_pressed() {
                state.zoom_slider_active = true;
                state.zoom_accumulator = 0.0;
            }
            if ui.just_released() {
                state.zoom_slider_active = false;
                state.zoom_slider_offset = 0.0;
                state.zoom_accumulator = 0.0;
            }
            ui.text("−", |t| t.font_size(16).color(theme.muted_text));
            // Inner track — visual
            ui.element()
                .id("zoom-track")
                .width(fixed!(inner_w))
                .height(fixed!(inner_h))
                .background_color(theme.surface_elevated)
                .corner_radius(inner_h / 2.0)
                .border(|b| b.all(1).color(ring_border))
                .empty();
            ui.text("+", |t| t.font_size(16).color(theme.muted_text));
            // Handle — activation + visual
            let zoom_pct = state.zoom_percent;
            zoom_handle(ui, state, handle_color, handle_w, handle_h, zoom_pct);
            // Slide tracking — active flag set by outer capsule or handle
            if state.zoom_slider_active {
                let (mx, _) = mouse_position();
                let dx = mx - slider_cx;
                let ox = dx.clamp(-max_offset, max_offset);
                state.zoom_slider_offset = ox;
                let zoom_speed = 1.5;
                state.zoom_accumulator += ox * zoom_speed / max_offset;
                if state.zoom_accumulator.abs() >= 1.0 {
                    let delta = state.zoom_accumulator as i32;
                    state.zoom_accumulator -= delta as f32;
                    adjust_zoom(state, delta);
                }
            }
        });
}

fn zoom_handle(ui: &mut Ui, state: &mut AppState, color: Color, w: f32, h: f32, pct: i32) {
    let zoom_text = format!("{pct}%");
    let border = Color::u_rgba(255, 255, 255, 25);
    let hx = state.zoom_slider_offset;
    ui.element()
        .id("zoom-handle")
        .width(fixed!(w))
        .height(fixed!(h))
        .floating(|f| {
            f.anchor((CenterX, CenterY), (CenterX, CenterY))
                .attach_parent()
                .offset((hx, 0.0))
                .z_index(11)
        })
        .background_color(color)
        .corner_radius(h / 2.0)
        .border(|b| b.all(1).color(border))
        .on_press(move |_, _| {})
        .layout(|l| l.align(CenterX, CenterY))
        .children(|ui| {
            if ui.just_pressed() {
                state.zoom_slider_active = true;
                state.zoom_accumulator = 0.0;
            }
            if ui.just_released() {
                state.zoom_slider_active = false;
                state.zoom_slider_offset = 0.0;
                state.zoom_accumulator = 0.0;
            }
            ui.text(&zoom_text, |t| {
                t.font_size(12).color(Color::u_rgba(220, 220, 225, 255))
            });
        });
}
// ── Private helpers ─────────────────────────────────────────────────

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
