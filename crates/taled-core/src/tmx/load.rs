mod convert;
mod prepare;
mod support;

use crate::error::Result;
use crate::model::Map;
use std::fs;
use std::path::{Path, PathBuf};

use convert::load_map_with_loader;
use prepare::prepare_map;
use support::normalize_path;

#[derive(Debug, Clone)]
struct PreparedMap {
    metadata: MapMetadata,
    resources: std::collections::BTreeMap<PathBuf, Vec<u8>>,
}

#[derive(Debug, Clone)]
struct MapMetadata {
    version: Option<String>,
    tiled_version: Option<String>,
    render_order: crate::model::RenderOrder,
    next_layer_id: Option<u32>,
    next_object_id: Option<u32>,
    tilesets: Vec<TilesetMetadata>,
    layers: Vec<LayerMetadata>,
    /// Object IDs whose shape is `<capsule>` (Tiled 1.12+, not in tiled crate).
    capsule_ids: std::collections::BTreeSet<u32>,
}

#[derive(Debug, Clone)]
struct TilesetMetadata {
    first_gid: u32,
    source: PathBuf,
    version: Option<String>,
    tiled_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayerKind {
    Tile,
    Object,
}

#[derive(Debug, Clone, Copy)]
struct LayerMetadata {
    kind: LayerKind,
    locked: bool,
}

#[derive(Debug, Clone)]
struct PreparedTileset {
    version: Option<String>,
    tiled_version: Option<String>,
    xml: String,
}

pub(crate) fn load_map(path: &Path) -> Result<Map> {
    let map_path = normalize_path(path);
    let xml = fs::read_to_string(&map_path)?;
    let prepared = prepare_map(&map_path, &xml, &|source_path| {
        Ok(fs::read_to_string(source_path)?)
    })?;

    load_map_with_loader(&map_path, xml.into_bytes(), prepared, &|requested_path| {
        Ok(fs::read(requested_path)?)
    })
}

pub(crate) fn load_map_from_str<F>(path: &Path, xml: &str, read_text: &F) -> Result<Map>
where
    F: Fn(&Path) -> Result<String>,
{
    let map_path = normalize_path(path);
    let prepared = prepare_map(&map_path, xml, read_text)?;

    load_map_with_loader(
        &map_path,
        xml.as_bytes().to_vec(),
        prepared,
        &|requested_path| Ok(read_text(requested_path)?.into_bytes()),
    )
}
