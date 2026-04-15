use serde::Deserialize;

/// Embedded UTDR map index (generated from open-utdr-maps).
static INDEX_JSON: &str = include_str!("../../../assets/utdr_index.json");

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UtdrIndex {
    #[allow(dead_code)]
    pub(crate) total_rooms: u32,
    pub(crate) repo: String,
    pub(crate) branch: String,
    pub(crate) games: std::collections::BTreeMap<String, UtdrGame>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UtdrGame {
    pub(crate) label: String,
    pub(crate) rooms: Vec<UtdrRoom>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct UtdrRoom {
    pub(crate) name: String,
    #[allow(dead_code)]
    pub(crate) path: String,
    pub(crate) size: u64,
}

/// Ordered list of game keys for consistent iteration.
pub(crate) const GAME_KEYS: &[&str] = &[
    "undertale",
    "deltarune_ch1",
    "deltarune_ch2",
    "deltarune_ch3",
    "deltarune_ch4",
];

/// Short display labels for game selector chips.
pub(crate) const GAME_SHORT_LABELS: &[&str] = &["UT", "DR1", "DR2", "DR3", "DR4"];

pub(crate) fn load_embedded_index() -> Option<UtdrIndex> {
    serde_json::from_str(INDEX_JSON).ok()
}
