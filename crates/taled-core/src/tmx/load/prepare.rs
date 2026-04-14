use super::{LayerKind, LayerMetadata, MapMetadata, PreparedMap, PreparedTileset, TilesetMetadata};
use crate::error::{EditorError, Result, unsupported};
use roxmltree::Node;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::support::{
    inject_root_attributes, normalize_path, parse_bool_attr, parse_document, parse_optional_u32,
    parse_render_order, parse_required_u32, reject_attr_if_present, reject_non_default_f32,
    required_attr,
};

pub(super) fn prepare_map<F>(map_path: &Path, xml: &str, read_text: &F) -> Result<PreparedMap>
where
    F: Fn(&Path) -> Result<String>,
{
    let document = parse_document(xml)?;
    let root = document.root_element();
    if root.tag_name().name() != "map" {
        return Err(EditorError::Invalid(
            "root element must be <map>".to_string(),
        ));
    }

    let orientation = required_attr(root, "orientation")?;
    if orientation != "orthogonal" {
        return Err(unsupported(
            "map.orientation",
            format!("unsupported orientation '{orientation}'"),
        ));
    }

    if root.attribute("infinite").unwrap_or("0") == "1" {
        return Err(unsupported(
            "map.infinite",
            "infinite maps are out of stage-1 scope",
        ));
    }

    reject_attr_if_present(root, "backgroundcolor", "map.background_color")?;
    reject_attr_if_present(root, "parallaxoriginx", "map.parallax_origin")?;
    reject_attr_if_present(root, "parallaxoriginy", "map.parallax_origin")?;
    reject_attr_if_present(root, "compressionlevel", "map.compression_level")?;

    let mut metadata = MapMetadata {
        version: root.attribute("version").map(ToOwned::to_owned),
        tiled_version: root.attribute("tiledversion").map(ToOwned::to_owned),
        render_order: parse_render_order(root.attribute("renderorder").unwrap_or("right-down"))?,
        next_layer_id: parse_optional_u32(root, "nextlayerid")?,
        next_object_id: parse_optional_u32(root, "nextobjectid")?,
        tilesets: Vec::new(),
        layers: Vec::new(),
        capsule_ids: std::collections::BTreeSet::new(),
    };
    let mut resources = BTreeMap::new();
    let map_root = map_path.parent().unwrap_or_else(|| Path::new("."));

    for child in root.children().filter(|node| node.is_element()) {
        match child.tag_name().name() {
            "properties" | "editorsettings" => {}
            "tileset" => prepare_tileset_reference(
                child,
                map_root,
                &mut metadata,
                &mut resources,
                read_text,
            )?,
            "layer" => {
                validate_tile_layer_node(child)?;
                metadata.layers.push(LayerMetadata {
                    kind: LayerKind::Tile,
                    locked: parse_bool_attr(child, "locked", false)?,
                });
            }
            "objectgroup" => {
                validate_object_layer_node(child)?;
                collect_capsule_ids(child, &mut metadata.capsule_ids)?;
                metadata.layers.push(LayerMetadata {
                    kind: LayerKind::Object,
                    locked: parse_bool_attr(child, "locked", false)?,
                });
            }
            "group" => {
                return Err(unsupported(
                    "layer.group",
                    "group layers are out of stage-1 scope",
                ));
            }
            "imagelayer" => {
                return Err(unsupported(
                    "layer.image",
                    "image layers are out of stage-1 scope",
                ));
            }
            value => {
                return Err(EditorError::Invalid(format!(
                    "unsupported <map> child '{value}'"
                )));
            }
        }
    }

    if metadata.tilesets.is_empty() {
        return Err(EditorError::Invalid(
            "map must reference at least one tileset".to_string(),
        ));
    }

    Ok(PreparedMap {
        metadata,
        resources,
    })
}

fn prepare_tileset_reference<F>(
    node: Node<'_, '_>,
    map_root: &Path,
    metadata: &mut MapMetadata,
    resources: &mut BTreeMap<PathBuf, Vec<u8>>,
    read_text: &F,
) -> Result<()>
where
    F: Fn(&Path) -> Result<String>,
{
    let first_gid = parse_required_u32(node, "firstgid")?;

    if let Some(source_str) = node.attribute("source") {
        // External tileset reference
        let source = PathBuf::from(source_str);
        let source_path = normalize_path(&map_root.join(&source));
        let tsx_xml = read_text(&source_path)?;
        let prepared_tileset = prepare_tileset(&tsx_xml)?;
        resources
            .entry(source_path)
            .or_insert_with(|| prepared_tileset.xml.clone().into_bytes());
        metadata.tilesets.push(TilesetMetadata {
            first_gid,
            source,
            version: prepared_tileset.version,
            tiled_version: prepared_tileset.tiled_version,
        });
    } else {
        // Embedded tileset — validate features, then let the tiled crate parse inline.
        validate_embedded_tileset_node(node)?;
        let synthetic = PathBuf::from(format!("_embedded_{first_gid}.tsx"));
        metadata.tilesets.push(TilesetMetadata {
            first_gid,
            source: synthetic,
            version: node.attribute("version").map(ToOwned::to_owned),
            tiled_version: node.attribute("tiledversion").map(ToOwned::to_owned),
        });
    }

    Ok(())
}

fn prepare_tileset(xml: &str) -> Result<PreparedTileset> {
    let document = parse_document(xml)?;
    let root = document.root_element();
    if root.tag_name().name() != "tileset" {
        return Err(EditorError::Invalid(
            "tileset root must be <tileset>".to_string(),
        ));
    }

    let spacing = parse_optional_u32(root, "spacing")?.unwrap_or(0);
    if spacing != 0 {
        return Err(unsupported(
            "tileset.spacing",
            "tileset spacing is out of stage-1 scope",
        ));
    }

    let margin = parse_optional_u32(root, "margin")?.unwrap_or(0);
    if margin != 0 {
        return Err(unsupported(
            "tileset.margin",
            "tileset margins are out of stage-1 scope",
        ));
    }

    let image_nodes: Vec<_> = root
        .children()
        .filter(|child| child.is_element() && child.tag_name().name() == "image")
        .collect();
    if image_nodes.len() > 1 {
        return Err(EditorError::Invalid(
            "tileset cannot contain multiple <image> nodes".to_string(),
        ));
    }

    let is_collection = image_nodes.is_empty();

    for child in root.children().filter(|node| node.is_element()) {
        match child.tag_name().name() {
            "image" | "tile" => {}
            // Collection-of-images tilesets commonly use <grid>; allow it.
            "grid" if is_collection => {}
            "properties" => {
                return Err(unsupported(
                    "tileset.properties",
                    "tileset properties are out of stage-1 scope",
                ));
            }
            "wangsets" | "terraintypes" | "transformations" | "tileoffset" | "grid" => {
                return Err(unsupported(
                    format!("tileset.{}", child.tag_name().name()),
                    "this tileset feature is out of stage-1 scope",
                ));
            }
            value => {
                return Err(EditorError::Invalid(format!(
                    "unsupported <tileset> child '{value}'"
                )));
            }
        }
    }

    // Collection-of-images tilesets have no atlas <image>; skip metric injection.
    let xml = if is_collection {
        xml.to_string()
    } else {
        inject_inferred_tileset_metrics(xml, root, image_nodes[0])?
    };
    Ok(PreparedTileset {
        version: root.attribute("version").map(ToOwned::to_owned),
        tiled_version: root.attribute("tiledversion").map(ToOwned::to_owned),
        xml,
    })
}

/// Validate an embedded `<tileset>` node inside the TMX (no external .tsx).
/// Checks the same feature gates as [`prepare_tileset`] but operates on the
/// roxmltree node directly, without re-parsing as a standalone document.
fn validate_embedded_tileset_node(node: Node<'_, '_>) -> Result<()> {
    let spacing = parse_optional_u32(node, "spacing")?.unwrap_or(0);
    if spacing != 0 {
        return Err(unsupported(
            "tileset.spacing",
            "tileset spacing is out of stage-1 scope",
        ));
    }
    let margin = parse_optional_u32(node, "margin")?.unwrap_or(0);
    if margin != 0 {
        return Err(unsupported(
            "tileset.margin",
            "tileset margins are out of stage-1 scope",
        ));
    }

    let mut image_count = 0u32;
    for child in node.children().filter(|n| n.is_element()) {
        match child.tag_name().name() {
            "image" => image_count += 1,
            // Allow <tile> elements including those with per-tile <image> children
            // (collection-of-images format).
            "tile" => {}
            "properties" | "wangsets" | "terraintypes" | "transformations" | "tileoffset"
            | "grid" => {}
            _ => {}
        }
    }

    if image_count > 1 {
        return Err(EditorError::Invalid(
            "tileset cannot contain multiple <image> nodes".to_string(),
        ));
    }

    Ok(())
}

fn validate_tile_layer_node(node: Node<'_, '_>) -> Result<()> {
    validate_common_layer_attrs(node)?;

    for child in node.children().filter(|child| child.is_element()) {
        match child.tag_name().name() {
            "data" | "properties" => {}
            value => {
                return Err(EditorError::Invalid(format!(
                    "unsupported <layer> child '{value}'"
                )));
            }
        }
    }

    Ok(())
}

fn validate_object_layer_node(node: Node<'_, '_>) -> Result<()> {
    validate_common_layer_attrs(node)?;
    reject_attr_if_present(node, "color", "layer.tint")?;
    reject_attr_if_present(node, "draworder", "object.draw_order")?;

    for child in node.children().filter(|child| child.is_element()) {
        match child.tag_name().name() {
            "object" => validate_object_node(child)?,
            "properties" => {}
            value => {
                return Err(EditorError::Invalid(format!(
                    "unsupported <objectgroup> child '{value}'"
                )));
            }
        }
    }

    Ok(())
}

/// Scan `<objectgroup>` children for objects containing `<capsule>` and record
/// their IDs so `convert_object` can produce `ObjectShape::Capsule` instead of
/// the `Rect` fallback that the `tiled` crate returns for unknown child tags.
fn collect_capsule_ids(
    group: Node<'_, '_>,
    ids: &mut std::collections::BTreeSet<u32>,
) -> Result<()> {
    for obj in group.children().filter(|n| n.is_element() && n.tag_name().name() == "object") {
        let has_capsule = obj
            .children()
            .any(|c| c.is_element() && c.tag_name().name() == "capsule");
        if has_capsule
            && let Some(id) = parse_optional_u32(obj, "id")?
        {
            ids.insert(id);
        }
    }
    Ok(())
}

fn validate_common_layer_attrs(node: Node<'_, '_>) -> Result<()> {
    reject_non_default_f32(node, "offsetx", 0.0, "layer.offset")?;
    reject_non_default_f32(node, "offsety", 0.0, "layer.offset")?;
    reject_non_default_f32(node, "parallaxx", 1.0, "layer.parallax_factor")?;
    reject_non_default_f32(node, "parallaxy", 1.0, "layer.parallax_factor")?;
    reject_attr_if_present(node, "tintcolor", "layer.tint")?;
    Ok(())
}

fn validate_object_node(node: Node<'_, '_>) -> Result<()> {
    if node.attribute("template").is_some() {
        return Err(unsupported(
            "object.template_instance",
            "template instances are out of stage-1 scope",
        ));
    }
    reject_non_default_f32(node, "rotation", 0.0, "object.rotate")?;

    for child in node.children().filter(|child| child.is_element()) {
        match child.tag_name().name() {
            "point" | "properties" | "ellipse" | "polygon" | "text" | "capsule" => {}
            "polyline" => {
                return Err(unsupported(
                    "object.polyline",
                    "polyline objects are out of stage-1 scope",
                ));
            }
            value => {
                return Err(EditorError::Invalid(format!(
                    "unsupported <object> child '{value}'"
                )));
            }
        }
    }

    Ok(())
}

fn inject_inferred_tileset_metrics(
    xml: &str,
    root: Node<'_, '_>,
    image: Node<'_, '_>,
) -> Result<String> {
    let mut attributes = Vec::new();

    let columns = match parse_optional_u32(root, "columns")? {
        Some(columns) => {
            if columns == 0 {
                return Err(EditorError::Invalid(
                    "tileset columns must be greater than zero".to_string(),
                ));
            }
            columns
        }
        None => {
            let image_width = parse_required_u32(image, "width")?;
            let tile_width = parse_required_u32(root, "tilewidth")?;
            let inferred = infer_columns(image_width, tile_width)?;
            attributes.push(("columns", inferred.to_string()));
            inferred
        }
    };

    if let Some(tile_count) = parse_optional_u32(root, "tilecount")? {
        if tile_count == 0 {
            return Err(EditorError::Invalid(
                "tileset tilecount must be greater than zero".to_string(),
            ));
        }
    } else {
        let image_width = parse_required_u32(image, "width")?;
        let image_height = parse_required_u32(image, "height")?;
        let tile_width = parse_required_u32(root, "tilewidth")?;
        let tile_height = parse_required_u32(root, "tileheight")?;
        let inferred =
            infer_tile_count(image_width, image_height, tile_width, tile_height, columns)?;
        attributes.push(("tilecount", inferred.to_string()));
    }

    if attributes.is_empty() {
        Ok(xml.to_string())
    } else {
        inject_root_attributes(xml, &attributes)
    }
}

fn infer_columns(image_width: u32, tile_width: u32) -> Result<u32> {
    if tile_width == 0 || !image_width.is_multiple_of(tile_width) {
        return Err(EditorError::Invalid(
            "cannot infer tileset columns from image width".to_string(),
        ));
    }
    Ok(image_width / tile_width)
}

fn infer_tile_count(
    image_width: u32,
    image_height: u32,
    tile_width: u32,
    tile_height: u32,
    columns: u32,
) -> Result<u32> {
    if tile_width == 0
        || tile_height == 0
        || !image_width.is_multiple_of(tile_width)
        || !image_height.is_multiple_of(tile_height)
    {
        return Err(EditorError::Invalid(
            "cannot infer tileset tilecount from image dimensions".to_string(),
        ));
    }
    Ok(columns * (image_height / tile_height))
}
