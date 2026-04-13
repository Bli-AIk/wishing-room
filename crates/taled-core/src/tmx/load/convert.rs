use super::{LayerKind, LayerMetadata, MapMetadata, PreparedMap, TilesetMetadata};
use crate::error::{EditorError, Result, unsupported};
use crate::model::{
    AnimationFrame, Layer, Map, MapObject, ObjectLayer, ObjectShape, Orientation, Property,
    PropertyValue, TileLayer, Tileset, TilesetImage, TilesetReference,
};
use std::collections::BTreeMap;
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};
use tiled::{
    LayerType, Loader, ObjectShape as TiledObjectShape, Orientation as TiledOrientation,
    PropertyValue as TiledPropertyValue, TileLayer as TiledTileLayer, Tileset as TiledTileset,
};

use super::support::{fallback_layer_name, normalize_path, relativize_child_path, to_io_error};

pub(super) fn load_map_with_loader<F>(
    map_path: &Path,
    map_bytes: Vec<u8>,
    prepared: PreparedMap,
    read_bytes: &F,
) -> Result<Map>
where
    F: Fn(&Path) -> Result<Vec<u8>>,
{
    let normalized_map_path = normalize_path(map_path);
    let loader_map_path = normalized_map_path.clone();
    let resources = prepared.resources.clone();

    let mut loader = Loader::with_reader(move |requested_path: &Path| -> io::Result<_> {
        let normalized = normalize_path(requested_path);
        let bytes = if normalized == normalized_map_path {
            map_bytes.clone()
        } else if let Some(bytes) = resources.get(&normalized) {
            bytes.clone()
        } else {
            read_bytes(&normalized).map_err(to_io_error)?
        };
        Ok(Cursor::new(bytes))
    });

    let raw_map = loader
        .load_tmx_map(&loader_map_path)
        .map_err(EditorError::from)?;
    convert_map(prepared.metadata, raw_map)
}

fn convert_map(metadata: MapMetadata, raw_map: tiled::Map) -> Result<Map> {
    if raw_map.orientation != TiledOrientation::Orthogonal {
        return Err(unsupported(
            "map.orientation",
            format!("unsupported orientation '{}'", raw_map.orientation),
        ));
    }
    if raw_map.infinite() {
        return Err(unsupported(
            "map.infinite",
            "infinite maps are out of stage-1 scope",
        ));
    }

    if raw_map.tilesets().len() != metadata.tilesets.len() {
        return Err(EditorError::Invalid(format!(
            "tileset metadata mismatch: expected {} but loader produced {}",
            metadata.tilesets.len(),
            raw_map.tilesets().len()
        )));
    }

    let tilesets = metadata
        .tilesets
        .iter()
        .zip(raw_map.tilesets().iter())
        .map(|(metadata, tileset)| convert_tileset(metadata, tileset))
        .collect::<Result<Vec<_>>>()?;

    if raw_map.layers().len() != metadata.layers.len() {
        return Err(EditorError::Invalid(format!(
            "layer metadata mismatch: expected {} but loader produced {}",
            metadata.layers.len(),
            raw_map.layers().len()
        )));
    }

    let layers = raw_map
        .layers()
        .zip(metadata.layers.iter())
        .map(
            |(layer, layer_metadata)| match (layer_metadata.kind, layer.layer_type()) {
                (LayerKind::Tile, LayerType::Tiles(tile_layer)) => {
                    convert_tile_layer(&metadata.tilesets, layer, tile_layer, *layer_metadata)
                }
                (LayerKind::Object, LayerType::Objects(object_layer)) => {
                    convert_object_layer(
                        layer,
                        object_layer,
                        *layer_metadata,
                        &metadata.tilesets,
                    )
                }
                _ => Err(EditorError::Invalid(
                    "layer type changed between validation and official parsing".to_string(),
                )),
            },
        )
        .collect::<Result<Vec<_>>>()?;

    let properties = convert_properties(&raw_map.properties, "map.properties")?;

    let next_layer_id = layers
        .iter()
        .map(Layer::id)
        .max()
        .unwrap_or(0)
        .max(metadata.next_layer_id.unwrap_or(1).saturating_sub(1))
        + 1;
    let next_object_id = layers
        .iter()
        .filter_map(|layer| layer.as_object())
        .flat_map(|layer| layer.objects.iter().map(|object| object.id))
        .max()
        .unwrap_or(0)
        .max(metadata.next_object_id.unwrap_or(1).saturating_sub(1))
        + 1;

    Ok(Map {
        version: metadata.version,
        tiled_version: metadata.tiled_version,
        orientation: Orientation::Orthogonal,
        render_order: metadata.render_order,
        width: raw_map.width,
        height: raw_map.height,
        tile_width: raw_map.tile_width,
        tile_height: raw_map.tile_height,
        next_layer_id,
        next_object_id,
        properties,
        tilesets,
        layers,
    })
}

fn convert_tileset(metadata: &TilesetMetadata, tileset: &TiledTileset) -> Result<TilesetReference> {
    // Collection-of-images tilesets have no atlas image; use a placeholder so
    // the map still loads.  The renderer skips tiles whose texture is missing.
    let image = if let Some(img) = tileset.image.as_ref() {
        let w = u32::try_from(img.width).map_err(|_| {
            EditorError::Invalid("tileset image width must be a positive integer".to_string())
        })?;
        let h = u32::try_from(img.height).map_err(|_| {
            EditorError::Invalid("tileset image height must be a positive integer".to_string())
        })?;
        TilesetImage {
            source: relativize_child_path(&tileset.source, &img.source),
            width: w,
            height: h,
        }
    } else {
        TilesetImage {
            source: PathBuf::from("_no_atlas"),
            width: 0,
            height: 0,
        }
    };

    let mut animations = BTreeMap::new();
    for (tile_id, tile) in tileset.tiles() {
        if let Some(frames) = &tile.animation {
            let converted: Vec<AnimationFrame> = frames
                .iter()
                .map(|f| AnimationFrame {
                    tile_id: f.tile_id,
                    duration_ms: f.duration,
                })
                .collect();
            if !converted.is_empty() {
                animations.insert(tile_id, converted);
            }
        }
    }

    Ok(TilesetReference {
        first_gid: metadata.first_gid,
        source: metadata.source.clone(),
        tileset: Tileset {
            version: metadata.version.clone(),
            tiled_version: metadata.tiled_version.clone(),
            name: tileset.name.clone(),
            tile_width: tileset.tile_width,
            tile_height: tileset.tile_height,
            tile_count: tileset.tilecount,
            columns: tileset.columns,
            image,
            animations,
        },
    })
}

fn convert_tile_layer(
    tilesets: &[TilesetMetadata],
    layer: tiled::Layer<'_>,
    tile_layer: TiledTileLayer<'_>,
    layer_metadata: LayerMetadata,
) -> Result<Layer> {
    let width = tile_layer
        .width()
        .ok_or_else(|| unsupported("map.infinite", "infinite maps are out of stage-1 scope"))?;
    let height = tile_layer
        .height()
        .ok_or_else(|| unsupported("map.infinite", "infinite maps are out of stage-1 scope"))?;

    let mut tiles = Vec::with_capacity((width * height) as usize);
    for y in 0..height {
        for x in 0..width {
            tiles.push(convert_cell_gid(
                tilesets,
                tile_layer.get_tile(x as i32, y as i32),
            )?);
        }
    }

    Ok(Layer::Tile(TileLayer {
        id: layer.id(),
        name: fallback_layer_name(&layer.name, "Tile Layer"),
        visible: layer.visible,
        locked: layer_metadata.locked,
        width,
        height,
        tiles,
        properties: convert_properties(&layer.properties, "layer.properties")?,
    }))
}

fn convert_cell_gid(
    tilesets: &[TilesetMetadata],
    tile: Option<tiled::LayerTile<'_>>,
) -> Result<u32> {
    let Some(tile) = tile else {
        return Ok(0);
    };

    let tileset = tilesets.get(tile.tileset_index()).ok_or_else(|| {
        EditorError::Invalid(format!(
            "unknown tileset index '{}' while converting tile layer",
            tile.tileset_index()
        ))
    })?;

    let mut gid = tileset.first_gid + tile.id();
    if tile.flip_h {
        gid |= 0x8000_0000;
    }
    if tile.flip_v {
        gid |= 0x4000_0000;
    }
    if tile.flip_d {
        gid |= 0x2000_0000;
    }
    Ok(gid)
}

fn convert_object_layer(
    layer: tiled::Layer<'_>,
    object_layer: tiled::ObjectLayer<'_>,
    layer_metadata: LayerMetadata,
    tileset_metas: &[TilesetMetadata],
) -> Result<Layer> {
    let objects = object_layer
        .objects()
        .map(|obj| convert_object(obj, tileset_metas))
        .collect::<Result<Vec<_>>>()?;

    Ok(Layer::Object(ObjectLayer {
        id: layer.id(),
        name: fallback_layer_name(&layer.name, "Object Layer"),
        visible: layer.visible,
        locked: layer_metadata.locked,
        objects,
        properties: convert_properties(&layer.properties, "layer.properties")?,
    }))
}

fn convert_object(
    object: tiled::Object<'_>,
    tileset_metas: &[TilesetMetadata],
) -> Result<MapObject> {
    let gid = object.tile_data().and_then(|td| {
        if let tiled::TilesetLocation::Map(idx) = td.tileset_location() {
            tileset_metas
                .get(*idx)
                .map(|meta| meta.first_gid + td.id())
        } else {
            None
        }
    });
    if object.rotation.abs() > f32::EPSILON {
        return Err(unsupported(
            "object.rotate",
            "attribute 'rotation' is out of stage-1 scope",
        ));
    }

    let (shape, width, height) = match &object.shape {
        TiledObjectShape::Rect { width, height } => (ObjectShape::Rectangle, *width, *height),
        TiledObjectShape::Point(_, _) => (ObjectShape::Point, 0.0, 0.0),
        TiledObjectShape::Ellipse { .. } => {
            return Err(unsupported(
                "object.ellipse",
                "ellipse objects are out of stage-1 scope",
            ));
        }
        TiledObjectShape::Polyline { .. } => {
            return Err(unsupported(
                "object.polyline",
                "polyline objects are out of stage-1 scope",
            ));
        }
        TiledObjectShape::Polygon { .. } => {
            return Err(unsupported(
                "object.polygon",
                "polygon objects are out of stage-1 scope",
            ));
        }
        TiledObjectShape::Text { .. } => {
            return Err(unsupported(
                "object.text",
                "text objects are out of stage-1 scope",
            ));
        }
    };

    Ok(MapObject {
        id: object.id(),
        name: object.name.clone(),
        visible: object.visible,
        x: object.x,
        y: object.y,
        width,
        height,
        shape,
        gid,
        properties: convert_properties(&object.properties, "object.properties")?,
    })
}

fn convert_properties(properties: &tiled::Properties, scope: &str) -> Result<Vec<Property>> {
    let mut converted = properties
        .iter()
        .map(|(name, value)| {
            Ok(Property {
                name: name.clone(),
                value: convert_property_value(value, scope)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    converted.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(converted)
}

fn convert_property_value(value: &TiledPropertyValue, scope: &str) -> Result<PropertyValue> {
    match value {
        TiledPropertyValue::StringValue(value) => Ok(PropertyValue::String(value.clone())),
        TiledPropertyValue::IntValue(value) => Ok(PropertyValue::Int(i64::from(*value))),
        TiledPropertyValue::FloatValue(value) => Ok(PropertyValue::Float(f64::from(*value))),
        TiledPropertyValue::BoolValue(value) => Ok(PropertyValue::Bool(*value)),
        TiledPropertyValue::ColorValue(_) => Err(unsupported(
            scope,
            "property type 'color' is out of stage-1 scope",
        )),
        TiledPropertyValue::FileValue(_) => Err(unsupported(
            scope,
            "property type 'file' is out of stage-1 scope",
        )),
        TiledPropertyValue::ObjectValue(_) => Err(unsupported(
            scope,
            "property type 'object' is out of stage-1 scope",
        )),
        TiledPropertyValue::ClassValue { .. } => Err(unsupported(
            scope,
            "property type 'class' is out of stage-1 scope",
        )),
    }
}
