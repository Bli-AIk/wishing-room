use ply_engine::prelude::*;

pub(crate) const DEFAULT_EMBEDDED_SAMPLE_PATH: &str = "stage1-basic/map.tmx";

#[allow(dead_code)]
pub(crate) struct EmbeddedSample {
    pub(crate) path: &'static str,
    pub(crate) title: &'static str,
    pub(crate) subtitle: &'static str,
    pub(crate) meta: &'static str,
    pub(crate) thumb: &'static GraphicAsset,
}

static THUMB_STAGE1: GraphicAsset = GraphicAsset::Bytes {
    file_name: "dashboard-stage1.png",
    data: include_bytes!("../../../assets/review/dashboard-stage1.png"),
};
static THUMB_THEATER: GraphicAsset = GraphicAsset::Bytes {
    file_name: "dashboard-theater.png",
    data: include_bytes!("../../../assets/review/dashboard-theater.png"),
};
static THUMB_FRONTIER: GraphicAsset = GraphicAsset::Bytes {
    file_name: "dashboard-frontier.png",
    data: include_bytes!("../../../assets/review/dashboard-frontier.png"),
};
static THUMB_RUINS: GraphicAsset = GraphicAsset::Bytes {
    file_name: "dashboard-ruins.png",
    data: include_bytes!("../../../assets/review/dashboard-ruins.png"),
};

static EMBEDDED_SAMPLES: [EmbeddedSample; 5] = [
    EmbeddedSample {
        path: "stage1-basic/map.tmx",
        title: "map.tmx",
        subtitle: "assets/samples/stage1-basic/map.tmx",
        meta: "Modified 2026-03-23 11:41 • 924 B • 6×5 @ 16 px",
        thumb: &THUMB_STAGE1,
    },
    EmbeddedSample {
        path: "test_obj/test.tmx",
        title: "Object Test",
        subtitle: "assets/samples/test_obj/test.tmx",
        meta: "30×20 @ 16 px • all object types",
        thumb: &THUMB_STAGE1,
    },
    EmbeddedSample {
        path: "ruins/ruins_3.tmx",
        title: "Ruins 3",
        subtitle: "assets/samples/ruins/ruins_3.tmx",
        meta: "37×12 @ 20 px • Souprune",
        thumb: &THUMB_RUINS,
    },
    EmbeddedSample {
        path: "maps/017-2.tmx",
        title: "Theater",
        subtitle: "assets/samples/tmwa/maps/017-2.tmx",
        meta: "Modified 2026-03-23 14:22 • 39.8 KB • 53×51 @ 32 px",
        thumb: &THUMB_THEATER,
    },
    EmbeddedSample {
        path: "maps/081-3.tmx",
        title: "Existential Frontier",
        subtitle: "assets/samples/tmwa/maps/081-3.tmx",
        meta: "Modified 2026-03-23 12:34 • 53.9 KB • 90×70 @ 32 px",
        thumb: &THUMB_FRONTIER,
    },
];

pub(crate) fn embedded_samples() -> &'static [EmbeddedSample] {
    &EMBEDDED_SAMPLES
}

pub(crate) fn embedded_sample(path: &str) -> Option<&'static EmbeddedSample> {
    embedded_samples().iter().find(|s| s.path == path)
}

#[allow(dead_code)]
pub(crate) fn embedded_sample_assets() -> [(&'static str, &'static [u8]); 42] {
    [
        (
            "stage1-basic/map.tmx",
            include_str!("../../../assets/samples/stage1-basic/map.tmx").as_bytes(),
        ),
        (
            "stage1-basic/terrain.tsx",
            include_str!("../../../assets/samples/stage1-basic/terrain.tsx").as_bytes(),
        ),
        (
            "stage1-basic/terrain.png",
            include_bytes!("../../../assets/samples/stage1-basic/terrain.png"),
        ),
        (
            "ruins/ruins_3.tmx",
            include_str!("../../../assets/samples/ruins/ruins_3.tmx").as_bytes(),
        ),
        (
            "ruins/ruins.tsx",
            include_str!("../../../assets/samples/ruins/ruins.tsx").as_bytes(),
        ),
        (
            "ruins/ruins_objects.tsx",
            include_str!("../../../assets/samples/ruins/ruins_objects.tsx").as_bytes(),
        ),
        (
            "ruins/tiles/ruins.png",
            include_bytes!("../../../assets/samples/ruins/tiles/ruins.png"),
        ),
        // Collection-of-images tile PNGs for ruins_objects.tsx
        (
            "ruins/tiles/objects/bigweb_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/bigweb_0.png"),
        ),
        (
            "ruins/tiles/objects/brand.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/brand.png"),
        ),
        (
            "ruins/tiles/objects/candydish_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/candydish_0.png"),
        ),
        (
            "ruins/tiles/objects/candydish2_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/candydish2_0.png"),
        ),
        (
            "ruins/tiles/objects/candydish2_1.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/candydish2_1.png"),
        ),
        (
            "ruins/tiles/objects/candydish_bad_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/candydish_bad_0.png"),
        ),
        (
            "ruins/tiles/objects/centeredhole_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/centeredhole_0.png"),
        ),
        (
            "ruins/tiles/objects/cheesetable_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/cheesetable_0.png"),
        ),
        (
            "ruins/tiles/objects/colorswitch_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/colorswitch_0.png"),
        ),
        (
            "ruins/tiles/objects/colorswitch_1.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/colorswitch_1.png"),
        ),
        (
            "ruins/tiles/objects/colorswitch_2.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/colorswitch_2.png"),
        ),
        (
            "ruins/tiles/objects/faceswitch_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/faceswitch_0.png"),
        ),
        (
            "ruins/tiles/objects/faceswitch_1.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/faceswitch_1.png"),
        ),
        (
            "ruins/tiles/objects/groundswitch1_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/groundswitch1_0.png"),
        ),
        (
            "ruins/tiles/objects/groundswitch1_1.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/groundswitch1_1.png"),
        ),
        (
            "ruins/tiles/objects/hole_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/hole_0.png"),
        ),
        (
            "ruins/tiles/objects/hole2_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/hole2_0.png"),
        ),
        (
            "ruins/tiles/objects/ribbon_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/ribbon_0.png"),
        ),
        (
            "ruins/tiles/objects/smallweb_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/smallweb_0.png"),
        ),
        (
            "ruins/tiles/objects/spiketile_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/spiketile_0.png"),
        ),
        (
            "ruins/tiles/objects/spiketile_1.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/spiketile_1.png"),
        ),
        (
            "ruins/tiles/objects/switch_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/switch_0.png"),
        ),
        (
            "ruins/tiles/objects/switch_1.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/switch_1.png"),
        ),
        (
            "ruins/tiles/objects/tornote_0.png",
            include_bytes!("../../../assets/samples/ruins/tiles/objects/tornote_0.png"),
        ),
        (
            "maps/017-2.tmx",
            include_str!("../../../assets/samples/tmwa/maps/017-2.tmx").as_bytes(),
        ),
        (
            "maps/081-3.tmx",
            include_str!("../../../assets/samples/tmwa/maps/081-3.tmx").as_bytes(),
        ),
        (
            "tilesets/collision.tsx",
            include_str!("../../../assets/samples/tmwa/tilesets/collision.tsx").as_bytes(),
        ),
        (
            "tilesets/woodland_indoor.tsx",
            include_str!("../../../assets/samples/tmwa/tilesets/woodland_indoor.tsx").as_bytes(),
        ),
        (
            "tilesets/icecave.tsx",
            include_str!("../../../assets/samples/tmwa/tilesets/icecave.tsx").as_bytes(),
        ),
        (
            "graphics/tiles/collision.png",
            include_bytes!("../../../assets/samples/tmwa/graphics/tiles/collision.png"),
        ),
        (
            "graphics/tiles/woodland_indoor.png",
            include_bytes!("../../../assets/samples/tmwa/graphics/tiles/woodland_indoor.png"),
        ),
        (
            "graphics/tiles/icecave.png",
            include_bytes!("../../../assets/samples/tmwa/graphics/tiles/icecave.png"),
        ),
        (
            "test_obj/test.tmx",
            include_str!("../../../assets/samples/test_obj/test.tmx").as_bytes(),
        ),
        (
            "test_obj/asset.tsx",
            include_str!("../../../assets/samples/test_obj/asset.tsx").as_bytes(),
        ),
        (
            "test_obj/asset.png",
            include_bytes!("../../../assets/samples/test_obj/asset.png"),
        ),
    ]
}
