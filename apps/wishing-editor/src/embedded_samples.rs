use std::sync::LazyLock;

use base64::Engine;

pub(crate) const DEFAULT_EMBEDDED_SAMPLE_PATH: &str = "stage1-basic/map.tmx";

pub(crate) struct EmbeddedSample {
    pub(crate) path: &'static str,
    pub(crate) title: &'static str,
    pub(crate) subtitle: &'static str,
    pub(crate) meta: &'static str,
    thumb_kind: ThumbKind,
}

#[derive(Clone, Copy)]
enum ThumbKind {
    Stage1,
    Theater,
    Frontier,
}

static DASHBOARD_STAGE1: LazyLock<String> =
    LazyLock::new(|| data_url(include_bytes!("../../../assets/review/dashboard-stage1.png")));
static DASHBOARD_THEATER: LazyLock<String> =
    LazyLock::new(|| data_url(include_bytes!("../../../assets/review/dashboard-theater.png")));
static DASHBOARD_FRONTIER: LazyLock<String> =
    LazyLock::new(|| data_url(include_bytes!("../../../assets/review/dashboard-frontier.png")));

static EMBEDDED_SAMPLES: [EmbeddedSample; 3] = [
    EmbeddedSample {
        path: "stage1-basic/map.tmx",
        title: "map.tmx",
        subtitle: "assets/samples/stage1-basic/map.tmx",
        meta: "Modified 2026-03-23 11:41 • 924 B • 6x5 @ 16 px",
        thumb_kind: ThumbKind::Stage1,
    },
    EmbeddedSample {
        path: "maps/017-2.tmx",
        title: "Theater",
        subtitle: "assets/samples/tmwa/maps/017-2.tmx",
        meta: "Modified 2026-03-23 14:22 • 39.8 KB • 53x51 @ 32 px",
        thumb_kind: ThumbKind::Theater,
    },
    EmbeddedSample {
        path: "maps/081-3.tmx",
        title: "Existential Frontier",
        subtitle: "assets/samples/tmwa/maps/081-3.tmx",
        meta: "Modified 2026-03-23 12:34 • 53.9 KB • 90x70 @ 32 px",
        thumb_kind: ThumbKind::Frontier,
    },
];

pub(crate) fn embedded_samples() -> &'static [EmbeddedSample] {
    &EMBEDDED_SAMPLES
}

pub(crate) fn embedded_sample(path: &str) -> Option<&'static EmbeddedSample> {
    embedded_samples().iter().find(|sample| sample.path == path)
}

pub(crate) fn embedded_sample_thumb(path: &str) -> &'static str {
    match embedded_sample(path).map(|sample| sample.thumb_kind) {
        Some(ThumbKind::Stage1) => DASHBOARD_STAGE1.as_str(),
        Some(ThumbKind::Theater) => DASHBOARD_THEATER.as_str(),
        Some(ThumbKind::Frontier) => DASHBOARD_FRONTIER.as_str(),
        None => "",
    }
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

fn data_url(bytes: &[u8]) -> String {
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}
