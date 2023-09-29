use std::collections::HashMap;

use serde::Deserialize;

const TREE_DATA: &[u8; 5996378] = include_bytes!("../resources/data.json");
pub const ORBIT_ANGLES: [i32; 16] = [
    0, 30, 45, 60, 90, 120, 135, 150, 180, 210, 225, 240, 270, 300, 315, 330,
];

#[derive(Deserialize, Default)]
pub struct Class {}

#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GroupBackground {
    pub image: String,
    pub is_half_image: Option<bool>,
}

#[derive(Deserialize, Default)]
pub struct Group {
    pub x: f32,
    pub y: f32,
    pub orbits: Vec<usize>,
    pub background: Option<GroupBackground>,
    pub nodes: Vec<String>,
}

#[derive(Deserialize)]
pub struct SpriteCoords {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

#[derive(Deserialize)]
pub struct Sprite {
    pub filename: String,
    pub w: usize,
    pub h: usize,
    pub coords: HashMap<String, SpriteCoords>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Node {
    pub skill: Option<usize>,
    pub name: Option<String>,
    pub icon: Option<String>,
    pub is_notable: Option<bool>,
    pub group: Option<usize>,
    pub orbit: Option<usize>,
    pub orbit_index: Option<usize>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Constants {
    pub orbit_radii: Vec<usize>,
}

#[derive(Deserialize, Default)]
pub struct TreeExport {
    pub tree: String,
    pub classes: Vec<Class>,
    pub groups: HashMap<String, Group>,
    pub nodes: HashMap<String, Node>,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub sprites: HashMap<String, HashMap<String, Sprite>>,
    pub constants: Constants,
}

impl TreeExport {
    pub fn new() -> Option<TreeExport> {
        match serde_json::from_str(&String::from_utf8_lossy(TREE_DATA)) {
            Ok(te) => Some(te),
            Err(e) => panic!("{}", e),
        }
    }
}
