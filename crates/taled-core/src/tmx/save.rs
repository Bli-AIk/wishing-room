use crate::error::Result;
use crate::model::{Layer, Map, ObjectLayer, ObjectShape, Property, TileLayer};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::fs;
use std::path::Path;

pub(crate) fn save_map(path: &Path, map: &Map) -> Result<()> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

    let mut map_tag = BytesStart::new("map");
    let version = map.version.as_deref().unwrap_or("1.10");
    let tiled_version = map.tiled_version.as_deref().unwrap_or("1.11.1");
    let width = map.width.to_string();
    let height = map.height.to_string();
    let tile_width = map.tile_width.to_string();
    let tile_height = map.tile_height.to_string();
    let next_layer_id = map.next_layer_id.to_string();
    let next_object_id = map.next_object_id.to_string();

    map_tag.push_attribute(("version", version));
    map_tag.push_attribute(("tiledversion", tiled_version));
    map_tag.push_attribute(("orientation", map.orientation.as_str()));
    map_tag.push_attribute(("renderorder", map.render_order.as_str()));
    map_tag.push_attribute(("width", width.as_str()));
    map_tag.push_attribute(("height", height.as_str()));
    map_tag.push_attribute(("tilewidth", tile_width.as_str()));
    map_tag.push_attribute(("tileheight", tile_height.as_str()));
    map_tag.push_attribute(("infinite", "0"));
    map_tag.push_attribute(("nextlayerid", next_layer_id.as_str()));
    map_tag.push_attribute(("nextobjectid", next_object_id.as_str()));
    writer.write_event(Event::Start(map_tag))?;

    write_properties(&mut writer, &map.properties)?;

    for tileset in &map.tilesets {
        let mut tag = BytesStart::new("tileset");
        let first_gid = tileset.first_gid.to_string();
        let source = tileset.source.to_string_lossy();
        tag.push_attribute(("firstgid", first_gid.as_str()));
        tag.push_attribute(("source", source.as_ref()));
        writer.write_event(Event::Empty(tag))?;
    }

    for layer in &map.layers {
        match layer {
            Layer::Tile(layer) => write_tile_layer(&mut writer, layer)?,
            Layer::Object(layer) => write_object_layer(&mut writer, layer)?,
        }
    }

    writer.write_event(Event::End(BytesEnd::new("map")))?;
    fs::write(path, writer.into_inner())?;
    Ok(())
}

fn write_tile_layer(writer: &mut Writer<Vec<u8>>, layer: &TileLayer) -> Result<()> {
    let mut tag = BytesStart::new("layer");
    let id = layer.id.to_string();
    let width = layer.width.to_string();
    let height = layer.height.to_string();
    tag.push_attribute(("id", id.as_str()));
    tag.push_attribute(("name", layer.name.as_str()));
    tag.push_attribute(("width", width.as_str()));
    tag.push_attribute(("height", height.as_str()));
    if !layer.visible {
        tag.push_attribute(("visible", "0"));
    }
    if layer.locked {
        tag.push_attribute(("locked", "1"));
    }
    writer.write_event(Event::Start(tag))?;
    write_properties(writer, &layer.properties)?;

    let mut data_tag = BytesStart::new("data");
    data_tag.push_attribute(("encoding", "csv"));
    writer.write_event(Event::Start(data_tag))?;

    let csv = build_csv_text(layer);
    writer.write_event(Event::Text(BytesText::new(csv.as_str())))?;
    writer.write_event(Event::End(BytesEnd::new("data")))?;
    writer.write_event(Event::End(BytesEnd::new("layer")))?;
    Ok(())
}

fn write_object_layer(writer: &mut Writer<Vec<u8>>, layer: &ObjectLayer) -> Result<()> {
    let mut tag = BytesStart::new("objectgroup");
    let id = layer.id.to_string();
    tag.push_attribute(("id", id.as_str()));
    tag.push_attribute(("name", layer.name.as_str()));
    if !layer.visible {
        tag.push_attribute(("visible", "0"));
    }
    if layer.locked {
        tag.push_attribute(("locked", "1"));
    }
    writer.write_event(Event::Start(tag))?;
    write_properties(writer, &layer.properties)?;

    for object in &layer.objects {
        let mut object_tag = BytesStart::new("object");
        let id = object.id.to_string();
        let x = format_f32(object.x);
        let y = format_f32(object.y);
        let width = format_f32(object.width);
        let height = format_f32(object.height);
        object_tag.push_attribute(("id", id.as_str()));
        if let Some(gid) = object.gid {
            let gid_str = gid.to_string();
            object_tag.push_attribute(("gid", gid_str.as_str()));
        }
        if !object.name.is_empty() {
            object_tag.push_attribute(("name", object.name.as_str()));
        }
        object_tag.push_attribute(("x", x.as_str()));
        object_tag.push_attribute(("y", y.as_str()));
        if !object.visible {
            object_tag.push_attribute(("visible", "0"));
        }
        if matches!(object.shape, ObjectShape::Rectangle) {
            object_tag.push_attribute(("width", width.as_str()));
            object_tag.push_attribute(("height", height.as_str()));
        }

        writer.write_event(Event::Start(object_tag))?;
        write_properties(writer, &object.properties)?;
        if object.is_point() {
            writer.write_event(Event::Empty(BytesStart::new("point")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("object")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("objectgroup")))?;
    Ok(())
}

fn write_properties(writer: &mut Writer<Vec<u8>>, properties: &[Property]) -> Result<()> {
    if properties.is_empty() {
        return Ok(());
    }

    writer.write_event(Event::Start(BytesStart::new("properties")))?;
    for property in properties {
        let mut tag = BytesStart::new("property");
        let value = property.value.as_editor_string();
        tag.push_attribute(("name", property.name.as_str()));
        if property.value.type_name() != "string" {
            tag.push_attribute(("type", property.value.type_name()));
        }
        tag.push_attribute(("value", value.as_str()));
        writer.write_event(Event::Empty(tag))?;
    }
    writer.write_event(Event::End(BytesEnd::new("properties")))?;
    Ok(())
}

fn build_csv_text(layer: &TileLayer) -> String {
    let mut rows = Vec::with_capacity(layer.height as usize);
    for y in 0..layer.height {
        let mut cols = Vec::with_capacity(layer.width as usize);
        for x in 0..layer.width {
            cols.push(layer.tile_at(x, y).unwrap_or(0).to_string());
        }
        rows.push(cols.join(","));
    }
    format!("\n{}\n", rows.join(",\n"))
}

fn format_f32(value: f32) -> String {
    let mut text = format!("{value:.3}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text.is_empty() {
        "0".to_string()
    } else {
        text
    }
}
