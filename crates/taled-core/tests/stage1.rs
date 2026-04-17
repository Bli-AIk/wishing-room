use std::fs;
use std::path::PathBuf;
use taled_core::{EditorSession, Layer};

fn sample_map_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .join("assets")
        .join("samples")
        .join("stage1-basic")
        .join("map.tmx")
}

fn tmwa_sample_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .join("assets")
        .join("samples")
        .join("tmwa")
        .join("maps")
        .join("081-3.tmx")
}

#[test]
fn loads_stage1_sample() {
    let session = EditorSession::load(sample_map_path()).expect("sample map should load");
    let document = session.document();

    assert_eq!(document.map.width, 6);
    assert_eq!(document.map.height, 5);
    assert_eq!(document.map.tilesets.len(), 1);
    assert_eq!(document.map.layers.len(), 2);

    let tile_layer = document.map.layers[0].as_tile().expect("tile layer");
    assert_eq!(tile_layer.tile_at(0, 0), Some(1));
    assert_eq!(tile_layer.tile_at(3, 0), Some(4));

    let object_layer = document.map.layers[1].as_object().expect("object layer");
    assert_eq!(object_layer.objects.len(), 2);
    assert!(matches!(
        object_layer.objects[1].shape,
        taled_core::ObjectShape::Point
    ));
}

#[test]
fn round_trips_supported_map() {
    let source = sample_map_path();
    let temp_dir = std::env::temp_dir().join("taled-stage1-tests");
    fs::create_dir_all(&temp_dir).expect("temp dir");
    let temp_map = temp_dir.join("roundtrip-map.tmx");
    let temp_tsx = temp_dir.join("terrain.tsx");

    fs::copy(&source, &temp_map).expect("copy map");
    fs::copy(source.with_file_name("terrain.tsx"), &temp_tsx).expect("copy tileset");
    fs::copy(
        source.with_file_name("terrain.png"),
        temp_dir.join("terrain.png"),
    )
    .expect("copy image");

    let mut session = EditorSession::load(&temp_map).expect("load copied map");
    session
        .edit(|document| {
            let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
            layer.set_tile(2, 2, 4)?;
            if let Layer::Object(layer) = &mut document.map.layers[1] {
                layer.objects[0].x = 32.0;
            }
            Ok(())
        })
        .expect("edit");
    session.save().expect("save roundtrip");

    let reloaded = EditorSession::load(&temp_map).expect("reload");
    let tile_layer = reloaded.document().map.layers[0]
        .as_tile()
        .expect("tile layer");
    let object_layer = reloaded.document().map.layers[1]
        .as_object()
        .expect("object layer");

    assert_eq!(tile_layer.tile_at(2, 2), Some(4));
    assert_eq!(object_layer.objects[0].x, 32.0);
}

#[test]
fn loads_map_with_embedded_tileset() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .join("assets/samples/embedded-basic/map.tmx");

    let session = EditorSession::load(&path).expect("embedded-tileset map should load");
    let document = session.document();

    assert_eq!(document.map.width, 6);
    assert_eq!(document.map.height, 5);
    assert_eq!(document.map.tilesets.len(), 1);
    assert_eq!(document.map.layers.len(), 2);

    let ts = &document.map.tilesets[0];
    assert_eq!(ts.tileset.name, "terrain");
    assert_eq!(ts.tileset.tile_count, 4);
    assert_eq!(ts.tileset.columns, 2);

    let tile_layer = document.map.layers[0].as_tile().expect("tile layer");
    assert_eq!(tile_layer.tile_at(0, 0), Some(1));
    assert_eq!(tile_layer.tile_at(3, 0), Some(4));

    let object_layer = document.map.layers[1].as_object().expect("object layer");
    assert_eq!(object_layer.objects.len(), 2);
}

#[test]
fn loads_map_with_collection_of_images_tileset() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate parent")
        .parent()
        .expect("workspace root")
        .join("assets/samples/collection-of-images/map.tmx");

    let session = EditorSession::load(&path).expect("collection-of-images map should load");
    let document = session.document();

    assert_eq!(document.map.width, 4);
    assert_eq!(document.map.height, 4);
    assert_eq!(document.map.tilesets.len(), 2);
    assert_eq!(document.map.layers.len(), 2);

    // First tileset: normal atlas
    let ts0 = &document.map.tilesets[0];
    assert_eq!(ts0.tileset.name, "terrain");
    assert_eq!(ts0.tileset.tile_count, 4);
    assert_eq!(ts0.tileset.columns, 2);
    assert_eq!(ts0.tileset.image.width, 40);

    // Second tileset: collection-of-images (placeholder image, columns=0)
    let ts1 = &document.map.tilesets[1];
    assert_eq!(ts1.tileset.name, "objects");
    assert_eq!(ts1.tileset.tile_count, 3);
    assert_eq!(ts1.tileset.columns, 0);
    assert_eq!(ts1.tileset.image.width, 0); // placeholder

    // Ground layer uses atlas tileset
    let ground = document.map.layers[0].as_tile().expect("tile layer");
    assert_eq!(ground.tile_at(0, 0), Some(1));
    assert_eq!(ground.tile_at(1, 0), Some(2));

    // Objects layer uses collection-of-images tileset
    let objects = document.map.layers[1]
        .as_tile()
        .expect("objects tile layer");
    assert_eq!(objects.tile_at(1, 0), Some(5)); // first object tile
    assert_eq!(objects.tile_at(3, 1), Some(6));
    assert_eq!(objects.tile_at(0, 3), Some(7));
    assert_eq!(objects.tile_at(0, 0), Some(0)); // empty cell
}

#[test]
fn session_history_tracks_undo_and_redo() {
    let mut session = EditorSession::load(sample_map_path()).expect("sample map should load");
    let original = session.document().map.layers[0]
        .as_tile()
        .expect("tile layer")
        .tile_at(1, 1)
        .expect("initial gid");
    let replacement = if original == 4 { 3 } else { 4 };

    assert!(!session.can_undo());
    assert!(!session.can_redo());

    session
        .edit(|document| {
            let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
            layer.set_tile(1, 1, replacement)?;
            Ok(())
        })
        .expect("edit should succeed");

    let edited = session.document().map.layers[0]
        .as_tile()
        .expect("tile layer")
        .tile_at(1, 1)
        .expect("edited gid");
    assert_eq!(edited, replacement);
    assert!(session.can_undo());
    assert!(!session.can_redo());

    assert!(session.undo());
    let undone = session.document().map.layers[0]
        .as_tile()
        .expect("tile layer")
        .tile_at(1, 1)
        .expect("undone gid");
    assert_eq!(undone, original);
    assert!(!session.can_undo());
    assert!(session.can_redo());

    assert!(session.redo());
    let redone = session.document().map.layers[0]
        .as_tile()
        .expect("tile layer")
        .tile_at(1, 1)
        .expect("redone gid");
    assert_eq!(redone, replacement);
    assert!(session.can_undo());
    assert!(!session.can_redo());
}

#[test]
fn history_batch_groups_multiple_tile_edits() {
    let mut session = EditorSession::load(sample_map_path()).expect("sample map should load");
    let original_a = session.document().map.layers[0]
        .as_tile()
        .expect("tile layer")
        .tile_at(0, 0)
        .expect("initial gid");
    let original_b = session.document().map.layers[0]
        .as_tile()
        .expect("tile layer")
        .tile_at(1, 0)
        .expect("initial gid");

    session.begin_history_batch();
    session
        .edit(|document| {
            let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
            layer.set_tile(0, 0, 4)?;
            Ok(())
        })
        .expect("first batched edit");
    session
        .edit(|document| {
            let layer = document.map.layers[0].as_tile_mut().expect("tile layer");
            layer.set_tile(1, 0, 3)?;
            Ok(())
        })
        .expect("second batched edit");

    assert!(session.finish_history_batch());
    assert!(session.can_undo());
    assert!(!session.can_redo());

    assert!(session.undo());
    let tile_layer = session.document().map.layers[0]
        .as_tile()
        .expect("tile layer");
    assert_eq!(tile_layer.tile_at(0, 0), Some(original_a));
    assert_eq!(tile_layer.tile_at(1, 0), Some(original_b));
    assert!(!session.can_undo());
    assert!(session.can_redo());
}

#[test]
fn loads_tmwa_sample_with_inferred_tileset_metrics() {
    let session = EditorSession::load(tmwa_sample_path()).expect("tmwa map should load");
    let document = session.document();

    assert_eq!(document.map.width, 90);
    assert_eq!(document.map.height, 70);
    assert_eq!(document.map.tilesets.len(), 2);
    assert_eq!(document.map.tilesets[0].tileset.tile_count, 2);
    assert_eq!(document.map.tilesets[1].tileset.columns, 16);
    assert_eq!(document.map.tilesets[1].tileset.tile_count, 128);
    assert_eq!(document.map.layers.len(), 5);
}

#[test]
fn loads_embedded_tmwa_sample() {
    let session = EditorSession::load_embedded(
        "maps/081-3.tmx",
        [
            (
                "maps/081-3.tmx",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../assets/samples/tmwa/maps/081-3.tmx"
                ))
                .as_bytes()
                .to_vec(),
            ),
            (
                "tilesets/collision.tsx",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../assets/samples/tmwa/tilesets/collision.tsx"
                ))
                .as_bytes()
                .to_vec(),
            ),
            (
                "tilesets/icecave.tsx",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../assets/samples/tmwa/tilesets/icecave.tsx"
                ))
                .as_bytes()
                .to_vec(),
            ),
            (
                "graphics/tiles/collision.png",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../assets/samples/tmwa/graphics/tiles/collision.png"
                ))
                .to_vec(),
            ),
            (
                "graphics/tiles/icecave.png",
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../assets/samples/tmwa/graphics/tiles/icecave.png"
                ))
                .to_vec(),
            ),
        ],
    )
    .expect("embedded tmwa sample should load");

    assert!(
        session
            .tileset_image_data_uri(1)
            .expect("embedded image uri")
            .starts_with("data:image/png;base64,")
    );
}

#[test]
fn rejects_infinite_maps() {
    let temp_dir = std::env::temp_dir().join("taled-stage1-tests");
    fs::create_dir_all(&temp_dir).expect("temp dir");
    let infinite_map = temp_dir.join("infinite-map.tmx");
    let tileset = temp_dir.join("terrain.tsx");

    fs::copy(sample_map_path().with_file_name("terrain.tsx"), &tileset).expect("copy tileset");
    fs::copy(
        sample_map_path().with_file_name("terrain.png"),
        temp_dir.join("terrain.png"),
    )
    .expect("copy image");

    fs::write(
        &infinite_map,
        r#"<?xml version="1.0" encoding="UTF-8"?>
<map version="1.10" tiledversion="1.11.1" orientation="orthogonal" renderorder="right-down" width="1" height="1" tilewidth="16" tileheight="16" infinite="1">
  <tileset firstgid="1" source="terrain.tsx"/>
</map>"#,
    )
    .expect("write infinite map");

    let error = EditorSession::load(infinite_map).expect_err("infinite map must be rejected");
    assert!(error.to_string().contains("map.infinite"));
}
