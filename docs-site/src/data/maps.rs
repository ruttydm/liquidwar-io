use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MapData {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub license: Option<String>,
    pub width: u32,
    pub height: u32,
}

pub fn all_maps() -> &'static Vec<MapData> {
    static MAPS: OnceLock<Vec<MapData>> = OnceLock::new();
    MAPS.get_or_init(|| {
        serde_json::from_str(include_str!("../../data/maps_data.json"))
            .expect("Failed to parse maps data")
    })
}

pub fn get_map(id: &str) -> Option<&'static MapData> {
    all_maps().iter().find(|m| m.id == id)
}

pub fn all_authors() -> Vec<(&'static str, usize)> {
    let mut counts = std::collections::HashMap::new();
    for m in all_maps() {
        if let Some(ref author) = m.author {
            *counts.entry(author.as_str()).or_insert(0usize) += 1;
        }
    }
    let mut v: Vec<_> = counts.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    v
}
