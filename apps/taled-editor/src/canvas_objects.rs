use std::collections::BTreeMap;

use ply_engine::prelude::*;
use taled_core::{MapObject, ObjectLayer, ObjectShape};

use crate::canvas::tile_transform;

/// Default colour for object outlines, matching Tiled's default (#a0a0a4).
const OBJ_COLOR: MacroquadColor = MacroquadColor::new(0.627, 0.627, 0.643, 1.0);
const OBJ_FILL: MacroquadColor = MacroquadColor::new(0.627, 0.627, 0.643, 0.33);
const SEL_COLOR: MacroquadColor = MacroquadColor::new(0.0, 0.6, 1.0, 1.0);
const SEL_FILL: MacroquadColor = MacroquadColor::new(0.0, 0.6, 1.0, 0.18);
const LINE_THICKNESS: f32 = 2.0;
const SEL_LINE_THICKNESS: f32 = 3.0;
const POINT_SIZE: f32 = 6.0;
const ELLIPSE_SEGMENTS: usize = 48;
const TEXT_SIZE: f32 = 12.0;

/// Convert a Tiled Y coordinate (top-down) to canvas Y (GL bottom-up render target).
fn fy(tiled_y: f32, z: f32, ch: f32) -> f32 {
    ch - tiled_y * z
}

/// Render all visible objects in a layer at canvas (zoomed) resolution.
///
/// `canvas_h` is `scaled_h` (= map pixel height × zoom) — needed because the
/// canvas render-target camera uses GL's Y-up convention.
pub(crate) fn render_object_layer(
    layer: &ObjectLayer,
    map: &taled_core::Map,
    textures: &BTreeMap<usize, Texture2D>,
    tile_textures: &BTreeMap<(usize, u32), Texture2D>,
    alpha: f32,
    zoom: f32,
    canvas_h: f32,
    selected_object: Option<u32>,
) {
    let color = MacroquadColor::new(OBJ_COLOR.r, OBJ_COLOR.g, OBJ_COLOR.b, alpha);
    let fill = MacroquadColor::new(OBJ_FILL.r, OBJ_FILL.g, OBJ_FILL.b, OBJ_FILL.a * alpha);

    for obj in &layer.objects {
        if !obj.visible {
            continue;
        }
        let is_selected = selected_object == Some(obj.id);
        if obj.gid.is_some() {
            draw_tile_object(obj, map, textures, tile_textures, alpha, zoom, canvas_h);
            if is_selected {
                draw_selection_border(obj, zoom, canvas_h);
            }
            continue;
        }
        let (c, f, lw) = if is_selected {
            (
                MacroquadColor::new(SEL_COLOR.r, SEL_COLOR.g, SEL_COLOR.b, alpha),
                MacroquadColor::new(SEL_FILL.r, SEL_FILL.g, SEL_FILL.b, SEL_FILL.a * alpha),
                SEL_LINE_THICKNESS,
            )
        } else {
            (color, fill, LINE_THICKNESS)
        };
        match &obj.shape {
            ObjectShape::Rectangle => draw_rect_outline(obj, zoom, canvas_h, c, lw),
            ObjectShape::Ellipse => draw_ellipse_outline(obj, zoom, canvas_h, c, lw),
            ObjectShape::Polygon { points } => {
                draw_polygon(obj, points, zoom, canvas_h, c, f, lw);
            }
            ObjectShape::Capsule => draw_capsule_outline(obj, zoom, canvas_h, c, lw),
            ObjectShape::Point => draw_point_marker(obj, zoom, canvas_h, c, lw),
            ObjectShape::Text { text, .. } => {
                draw_text_object(obj, text, zoom, canvas_h, c);
            }
        }
    }
}

fn draw_rect_outline(obj: &MapObject, z: f32, ch: f32, color: MacroquadColor, lw: f32) {
    let y = fy(obj.y + obj.height, z, ch);
    draw_rectangle_lines(obj.x * z, y, obj.width * z, obj.height * z, lw, color);
}

fn draw_ellipse_outline(obj: &MapObject, z: f32, ch: f32, color: MacroquadColor, lw: f32) {
    let cx = (obj.x + obj.width / 2.0) * z;
    let cy = fy(obj.y + obj.height / 2.0, z, ch);
    let rx = obj.width / 2.0 * z;
    let ry = obj.height / 2.0 * z;

    for i in 0..ELLIPSE_SEGMENTS {
        let a0 = std::f32::consts::TAU * i as f32 / ELLIPSE_SEGMENTS as f32;
        let a1 = std::f32::consts::TAU * (i + 1) as f32 / ELLIPSE_SEGMENTS as f32;
        draw_line(
            cx + rx * a0.cos(),
            cy + ry * a0.sin(),
            cx + rx * a1.cos(),
            cy + ry * a1.sin(),
            lw,
            color,
        );
    }
}

fn draw_polygon(
    obj: &MapObject,
    points: &[(f32, f32)],
    z: f32,
    ch: f32,
    color: MacroquadColor,
    fill: MacroquadColor,
    lw: f32,
) {
    if points.len() < 2 {
        return;
    }
    if points.len() >= 3 {
        let v0 = Vec2::new((obj.x + points[0].0) * z, fy(obj.y + points[0].1, z, ch));
        for i in 1..points.len() - 1 {
            let v1 = Vec2::new((obj.x + points[i].0) * z, fy(obj.y + points[i].1, z, ch));
            let v2 = Vec2::new(
                (obj.x + points[i + 1].0) * z,
                fy(obj.y + points[i + 1].1, z, ch),
            );
            draw_triangle(v0, v1, v2, fill);
        }
    }
    for i in 0..points.len() {
        let (x1, y1) = points[i];
        let (x2, y2) = points[(i + 1) % points.len()];
        draw_line(
            (obj.x + x1) * z,
            fy(obj.y + y1, z, ch),
            (obj.x + x2) * z,
            fy(obj.y + y2, z, ch),
            lw,
            color,
        );
    }
}

fn draw_capsule_outline(obj: &MapObject, z: f32, ch: f32, color: MacroquadColor, lw: f32) {
    let w = obj.width;
    let h = obj.height;
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    if h >= w {
        let r = w / 2.0;
        let cx = (obj.x + r) * z;
        let straight = h - w;
        let top = fy(obj.y + r, z, ch);
        let bot = fy(obj.y + r + straight, z, ch);
        let lx = obj.x * z;
        let rx = (obj.x + w) * z;
        draw_line(lx, top, lx, bot, lw, color);
        draw_line(rx, top, rx, bot, lw, color);
        // Top dome arcs upward (+Y in canvas); bottom dome arcs downward.
        draw_semicircle(cx, top, r * z, std::f32::consts::PI, 0.0, color);
        draw_semicircle(cx, bot, r * z, 0.0, -std::f32::consts::PI, color);
    } else {
        let r = h / 2.0;
        let cy = fy(obj.y + r, z, ch);
        let straight = w - h;
        let left = (obj.x + r) * z;
        let right = (obj.x + r + straight) * z;
        let ty = fy(obj.y, z, ch);
        let by = fy(obj.y + h, z, ch);
        draw_line(left, ty, right, ty, lw, color);
        draw_line(left, by, right, by, lw, color);
        use std::f32::consts::{FRAC_PI_2, PI};
        draw_semicircle(left, cy, r * z, FRAC_PI_2, PI + FRAC_PI_2, color);
        draw_semicircle(right, cy, r * z, -FRAC_PI_2, FRAC_PI_2, color);
    }
}

/// Draw a semicircular arc. Uses `cy + r·sin(t)` so that positive sin points
/// upward in the Y-up canvas coordinate system.
fn draw_semicircle(cx: f32, cy: f32, r: f32, start: f32, end: f32, color: MacroquadColor) {
    let segs = ELLIPSE_SEGMENTS / 2;
    for i in 0..segs {
        let t0 = start + (end - start) * i as f32 / segs as f32;
        let t1 = start + (end - start) * (i + 1) as f32 / segs as f32;
        draw_line(
            cx + r * t0.cos(),
            cy + r * t0.sin(),
            cx + r * t1.cos(),
            cy + r * t1.sin(),
            LINE_THICKNESS,
            color,
        );
    }
}

fn draw_point_marker(obj: &MapObject, z: f32, ch: f32, color: MacroquadColor, lw: f32) {
    let x = obj.x * z;
    let y = fy(obj.y, z, ch);
    let s = POINT_SIZE;
    draw_line(x, y + s, x + s, y, lw, color);
    draw_line(x + s, y, x, y - s, lw, color);
    draw_line(x, y - s, x - s, y, lw, color);
    draw_line(x - s, y, x, y + s, lw, color);
    draw_line(x - s * 0.4, y, x + s * 0.4, y, lw, color);
    draw_line(x, y - s * 0.4, x, y + s * 0.4, lw, color);
}

fn draw_text_object(obj: &MapObject, text: &str, z: f32, ch: f32, color: MacroquadColor) {
    let size = TEXT_SIZE * z;
    draw_text(text, obj.x * z, fy(obj.y, z, ch), size, color);
}

/// Draw a highlight border around a tile object (sprite) when selected.
fn draw_selection_border(obj: &MapObject, z: f32, ch: f32) {
    let dx = obj.x * z;
    let dy = fy(obj.y, z, ch);
    let dw = obj.width * z;
    let dh = obj.height * z;
    draw_rectangle_lines(dx, dy, dw, dh, SEL_LINE_THICKNESS, SEL_COLOR);
}

fn draw_tile_object(
    obj: &MapObject,
    map: &taled_core::Map,
    textures: &BTreeMap<usize, Texture2D>,
    tile_textures: &BTreeMap<(usize, u32), Texture2D>,
    alpha: f32,
    z: f32,
    ch: f32,
) {
    let raw_gid = match obj.gid {
        Some(g) => g,
        None => return,
    };
    let Some(tile_ref) = map.tile_reference_for_gid(raw_gid) else {
        return;
    };

    let ts = &tile_ref.tileset.tileset;

    // Determine the texture and source rect. For collection-of-images tilesets
    // each tile has its own texture; for atlas tilesets we crop the atlas.
    let (texture, sx, sy, sw, sh) =
        if let Some(tex) = tile_textures.get(&(tile_ref.tileset_index, tile_ref.local_id)) {
            (tex, 0.0, 0.0, tex.width(), tex.height())
        } else if let Some(tex) = textures.get(&tile_ref.tileset_index) {
            let cols = (ts.image.width / ts.tile_width).max(1);
            let src_col = tile_ref.local_id % cols;
            let src_row = tile_ref.local_id / cols;
            let sx = src_col as f32 * ts.tile_width as f32;
            let sy = src_row as f32 * ts.tile_height as f32;
            (tex, sx, sy, ts.tile_width as f32, ts.tile_height as f32)
        } else {
            return;
        };

    let (flip_h, flip_v, flip_d) = taled_core::tile_flip_flags(raw_gid);
    let (rotation, flip_x, flip_y) = tile_transform(flip_h, flip_v, flip_d);

    // Tile objects anchor at bottom-left in TMX.
    // In canvas (Y-up): bottom of tile = fy(obj.y); tile extends upward by dh.
    let dx = obj.x * z;
    let dy = fy(obj.y, z, ch);
    let dw = obj.width * z;
    let dh = obj.height * z;
    let color = MacroquadColor::new(1.0, 1.0, 1.0, alpha);

    // In Y-up canvas draw_texture_ex maps texture (0,0) to (dx, dy) = quad bottom,
    // placing the image top-left at the visual bottom. Toggle flip_y to compensate.
    draw_texture_ex(
        texture,
        dx,
        dy,
        color,
        DrawTextureParams {
            source: Some(Rect::new(sx, sy, sw, sh)),
            dest_size: Some(Vec2::new(dw, dh)),
            rotation,
            flip_x,
            flip_y: !flip_y,
            pivot: Some(Vec2::new(dx + dw / 2.0, dy + dh / 2.0)),
        },
    );
}

/// Hit-test objects in an object layer.
/// `world_x`, `world_y` are in Tiled map coordinates (top-down, unzoomed).
/// Returns the id of the topmost object containing the point (last in draw order).
pub(crate) fn hit_test_object(layer: &ObjectLayer, world_x: f32, world_y: f32) -> Option<u32> {
    let mut hit: Option<u32> = None;
    for obj in &layer.objects {
        if !obj.visible {
            continue;
        }
        if obj_contains_point(obj, world_x, world_y) {
            hit = Some(obj.id);
        }
    }
    hit
}

/// Test if a point (in Tiled world coords) falls inside the object's bounds.
fn obj_contains_point(obj: &MapObject, world_x: f32, world_y: f32) -> bool {
    if obj.gid.is_some() {
        // Tile objects anchor at bottom-left in TMX: y is the bottom.
        return world_x >= obj.x
            && world_x <= obj.x + obj.width
            && world_y >= obj.y - obj.height
            && world_y <= obj.y;
    }
    match &obj.shape {
        ObjectShape::Point => {
            let dx = world_x - obj.x;
            let dy = world_y - obj.y;
            (dx * dx + dy * dy) < POINT_SIZE * POINT_SIZE * 4.0
        }
        ObjectShape::Ellipse => {
            let cx = obj.x + obj.width / 2.0;
            let cy = obj.y + obj.height / 2.0;
            let rx = obj.width / 2.0;
            let ry = obj.height / 2.0;
            if rx <= 0.0 || ry <= 0.0 {
                return false;
            }
            let dx = (world_x - cx) / rx;
            let dy = (world_y - cy) / ry;
            dx * dx + dy * dy <= 1.0
        }
        ObjectShape::Rectangle | ObjectShape::Capsule | ObjectShape::Text { .. } => {
            world_x >= obj.x
                && world_x <= obj.x + obj.width
                && world_y >= obj.y
                && world_y <= obj.y + obj.height
        }
        ObjectShape::Polygon { points } => {
            point_in_polygon(world_x - obj.x, world_y - obj.y, points)
        }
    }
}

/// Point-in-polygon test using ray casting algorithm.
fn point_in_polygon(px: f32, py: f32, points: &[(f32, f32)]) -> bool {
    let n = points.len();
    if n < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = points[i];
        let (xj, yj) = points[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}
