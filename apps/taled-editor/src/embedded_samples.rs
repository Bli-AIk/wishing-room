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

static EMBEDDED_SAMPLES: [EmbeddedSample; 3] = [
    EmbeddedSample {
        path: "stage1-basic/map.tmx",
        title: "map.tmx",
        subtitle: "assets/samples/stage1-basic/map.tmx",
        meta: "Modified 2026-03-23 11:41 · 924 B · 6×5 @ 16 px",
        thumb: &THUMB_STAGE1,
    },
    EmbeddedSample {
        path: "maps/017-2.tmx",
        title: "Theater",
        subtitle: "assets/samples/tmwa/maps/017-2.tmx",
        meta: "Modified 2026-03-23 14:22 · 39.8 KB · 53×51 @ 32 px",
        thumb: &THUMB_THEATER,
    },
    EmbeddedSample {
        path: "maps/081-3.tmx",
        title: "Existential Frontier",
        subtitle: "assets/samples/tmwa/maps/081-3.tmx",
        meta: "Modified 2026-03-23 12:34 · 53.9 KB · 90×70 @ 32 px",
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
pub(crate) fn embedded_sample_assets() -> [(&'static str, &'static [u8]); 11] {
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
    ]
}
