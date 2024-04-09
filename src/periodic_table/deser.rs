use serde::Deserialize;

use super::ElementNumber;

#[derive(Deserialize, Clone)]
pub struct Element {
    pub name: String,
    pub symbol: String,
    pub number: ElementNumber,
    pub atomic_mass: f64,
    pub density: Option<f64>
}

#[derive(Deserialize)]
pub(super) struct Table {
    pub elements: Vec<Element>
}