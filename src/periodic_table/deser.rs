use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Element {
    pub name: String,
    pub symbol: String,
    pub number: u32,
    pub atomic_mass: f64,
    pub density: Option<f64>
}

#[derive(Deserialize)]
pub(super) struct Table {
    pub elements: Vec<Element>
}