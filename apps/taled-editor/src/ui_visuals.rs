use std::collections::BTreeMap;

use taled_core::{EditorDocument, MapObject, ObjectShape};

use crate::app_state::PaletteTile;

const OBJECT_MAIN_HEX: &str = "#808080";
const OBJECT_FILL_RGBA: &str = "rgba(128,128,128,0.168)";
const OBJECT_SHADOW_RGBA: &str = "rgba(0,0,0,0.92)";
const OBJECT_SELECTED_RGBA: &str = "rgba(162,168,176,0.30)";
const OBJECT_SELECTED_STROKE_RGBA: &str = "rgba(168,174,182,0.68)";
const PALETTE_PREVIEW_SIZE: f32 = 44.0;
const PALETTE_INSET: f32 = 4.0;
const POINT_MARKER_WIDTH: f32 = 20.0;
const POINT_MARKER_HEIGHT: f32 = 30.0;

pub(crate) fn palette_tile_style(
    document: &EditorDocument,
    image_cache: &BTreeMap<usize, String>,
    tile: &PaletteTile,
) -> String {
    let Some(reference) = document.map.tile_reference_for_gid(tile.gid) else {
        return String::new();
    };
    let Some(image) = image_cache.get(&tile.tileset_index) else {
        return String::new();
    };

    let columns = reference.tileset.tileset.columns.max(1);
    let tile_width = reference.tileset.tileset.tile_width as f32;
    let tile_height = reference.tileset.tileset.tile_height as f32;
    let atlas_width = reference.tileset.tileset.image.width as f32;
    let atlas_height = reference.tileset.tileset.image.height as f32;
    let source_x = (tile.local_id % columns) as f32 * tile_width;
    let source_y = (tile.local_id / columns) as f32 * tile_height;

    let preview_box = PALETTE_PREVIEW_SIZE - PALETTE_INSET * 2.0;
    let scale = (preview_box / tile_width)
        .min(preview_box / tile_height)
        .max(1.0);
    let rendered_width = tile_width * scale;
    let rendered_height = tile_height * scale;
    let inset_x = (PALETTE_PREVIEW_SIZE - rendered_width) / 2.0;
    let inset_y = (PALETTE_PREVIEW_SIZE - rendered_height) / 2.0;
    let tile_preview = tile_preview_data_uri(
        image,
        atlas_width,
        atlas_height,
        source_x,
        source_y,
        tile_width,
        tile_height,
        inset_x,
        inset_y,
        rendered_width,
        rendered_height,
    );

    format!(
        "background-image:{tile_preview};background-position:center;background-size:100% 100%;background-repeat:no-repeat;",
    )
}

pub(crate) fn object_overlay_style(
    object: &MapObject,
    selectable: bool,
    selected: bool,
    zoom: f32,
) -> String {
    let pointer_events = if selectable { "auto" } else { "none" };
    match object.shape {
        ObjectShape::Rectangle => rectangle_overlay_style(object, pointer_events, selected),
        ObjectShape::Point => point_overlay_style(object, pointer_events, selected, zoom),
    }
}

pub(crate) fn object_icon_style(shape: &ObjectShape) -> String {
    let image = match shape {
        ObjectShape::Rectangle => rectangle_icon_data_uri(),
        ObjectShape::Point => point_marker_data_uri(),
    };

    format!(
        "background-image:{image};background-size:contain;background-repeat:no-repeat;background-position:center;"
    )
}

fn rectangle_overlay_style(object: &MapObject, pointer_events: &str, selected: bool) -> String {
    let selected_outline = if selected {
        format!(
            "outline:0.5px solid {OBJECT_SELECTED_STROKE_RGBA};outline-offset:0;box-shadow:0 0 0 0.5px rgba(255,255,255,0.06);"
        )
    } else {
        String::new()
    };

    format!(
        concat!(
            "left:{}px;top:{}px;width:{}px;height:{}px;pointer-events:{};",
            "border:0.5px solid {};background:{};box-shadow:0.5px 0.5px 0 {};",
            "{}"
        ),
        object.x,
        object.y,
        object.width.max(1.0),
        object.height.max(1.0),
        pointer_events,
        OBJECT_MAIN_HEX,
        OBJECT_FILL_RGBA,
        OBJECT_SHADOW_RGBA,
        selected_outline,
    )
}

fn point_overlay_style(
    object: &MapObject,
    pointer_events: &str,
    selected: bool,
    zoom: f32,
) -> String {
    let mut filters = vec![format!("drop-shadow(0.5px 0.5px 0 {OBJECT_SHADOW_RGBA})")];
    if selected {
        filters.push(format!("drop-shadow(0 0 2px {OBJECT_SELECTED_RGBA})"));
    }

    format!(
        concat!(
            "left:{}px;top:{}px;width:{}px;height:{}px;pointer-events:{};",
            "background-image:{};background-size:contain;background-repeat:no-repeat;background-position:center;",
            "transform-origin:center bottom;transform:scale({});filter:{};"
        ),
        object.x - POINT_MARKER_WIDTH / 2.0,
        object.y - POINT_MARKER_HEIGHT,
        POINT_MARKER_WIDTH,
        POINT_MARKER_HEIGHT,
        pointer_events,
        point_marker_data_uri(),
        (1.0 / zoom.max(0.01)),
        filters.join(" "),
    )
}

fn rectangle_icon_data_uri() -> String {
    svg_data_uri(
        r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 20 20'>
<rect x='3' y='3' width='14' height='14' fill='rgba(128,128,128,0.168)' stroke='#808080' stroke-width='1'/>
</svg>"#,
    )
}

fn point_marker_data_uri() -> String {
    svg_data_uri(
        r#"<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 20 30'>
<path fill='rgba(128,128,128,0.168)' fill-rule='evenodd' stroke='#808080' stroke-width='1' d='M10 1C5.03 1 1 5.03 1 10c0 5.06 3.68 8.43 9 18 5.32-9.57 9-12.94 9-18 0-4.97-4.03-9-9-9Zm0 4.75a4.25 4.25 0 1 1 0 8.5a4.25 4.25 0 0 1 0-8.5Z'/>
</svg>"#,
    )
}

#[allow(clippy::too_many_arguments)]
fn tile_preview_data_uri(
    image: &str,
    atlas_width: f32,
    atlas_height: f32,
    source_x: f32,
    source_y: f32,
    tile_width: f32,
    tile_height: f32,
    inset_x: f32,
    inset_y: f32,
    rendered_width: f32,
    rendered_height: f32,
) -> String {
    svg_data_uri(&format!(
        concat!(
            "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 {size} {size}' shape-rendering='crispEdges'>",
            "<rect width='{size}' height='{size}' rx='10' fill='transparent'/>",
            "<svg x='{inset_x}' y='{inset_y}' width='{rendered_width}' height='{rendered_height}' ",
            "viewBox='{source_x} {source_y} {tile_width} {tile_height}' preserveAspectRatio='none'>",
            "<image href='{image}' width='{atlas_width}' height='{atlas_height}'/>",
            "</svg>",
            "</svg>"
        ),
        size = PALETTE_PREVIEW_SIZE,
        inset_x = inset_x,
        inset_y = inset_y,
        rendered_width = rendered_width,
        rendered_height = rendered_height,
        source_x = source_x,
        source_y = source_y,
        tile_width = tile_width,
        tile_height = tile_height,
        image = image,
        atlas_width = atlas_width,
        atlas_height = atlas_height,
    ))
}

fn svg_data_uri(svg: &str) -> String {
    let encoded = svg
        .replace('%', "%25")
        .replace('&', "%26")
        .replace('#', "%23")
        .replace('<', "%3C")
        .replace('>', "%3E")
        .replace('"', "'")
        .replace('\n', "")
        .replace(' ', "%20");
    format!("url(\"data:image/svg+xml;utf8,{encoded}\")")
}
