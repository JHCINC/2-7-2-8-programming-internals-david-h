//! Using schema and data fetched from:
//!
//! [https://github.com/Bowserinator/Periodic-Table-JSON]
//!
//! Data licensed under [CC BY-SA 3.0 DEED](https://creativecommons.org/licenses/by-sa/3.0/)
//! from [Bowserinator](https://github.com/Bowserinator) on GitHub.
use std::{collections::HashMap, num::NonZeroU32};

use slotmap::{new_key_type, SlotMap};

mod deser;
pub type ElementNumber = NonZeroU32;

new_key_type! { struct ElementKey; }

/// The data of a periodic table.
pub struct PeriodicTable {
    by_name: HashMap<String, ElementKey>,
    by_number: HashMap<ElementNumber, ElementKey>,
    by_symbol: HashMap<String, ElementKey>,
    elements: SlotMap<ElementKey, deser::Element>,
}

impl PeriodicTable {
    /// Gets an element by symbol.
    ///
    /// Returns `None` if there is no element
    /// with the given symbol.
    pub fn by_symbol(&self, sym: &str) -> Option<&deser::Element> {
        let idx = self.by_symbol.get(sym)?;
        Some(&self.elements[*idx])
    }

    /// Gets an element by number.
    ///
    /// Returns `None` if there is no element
    /// with the given number.
    pub fn by_number(&self, num: u32) -> Option<&deser::Element> {
        let idx = self.by_number.get(&NonZeroU32::new(num)?)?;
        Some(&self.elements[*idx])
    }

    /// Gets an element by name.
    ///
    /// Returns `None` if there is no element
    /// with the given name.
    pub fn by_name(&self, name: &str) -> Option<&deser::Element> {
        let idx = self.by_name.get(name)?;
        Some(&self.elements[*idx])
    }

    /// Accepts JSON data in the schema
    /// depicted in the [deser] module.
    pub fn from_json(s: impl std::io::Read) -> anyhow::Result<Self> {
        let table: deser::Table = serde_json::from_reader(s)?;

        let mut elements = SlotMap::with_key();

        let mut by_name = HashMap::default();
        let mut by_number = HashMap::default();
        let mut by_symbol = HashMap::default();

        for element_entry in table.elements {
            let index = elements.insert(element_entry.clone());

            by_name.insert(element_entry.name, index);
            by_number.insert(element_entry.number, index);
            by_symbol.insert(element_entry.symbol, index);
        }

        Ok(Self {
            elements,
            by_name,
            by_number,
            by_symbol,
        })
    }
}

/// Values printable with a PeriodicTable.
pub trait TablePrintable {
    fn fmt(&self, t: &PeriodicTable, f: &mut impl std::fmt::Write) -> std::fmt::Result;
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::PeriodicTable;

    #[test]
    fn assert_basic_elements() {
        let p = PeriodicTable::from_json(File::open("./PeriodicTableJSON.json").unwrap()).unwrap();

        assert_eq!(p.by_number(1).unwrap().name, "Hydrogen");
        assert_eq!(p.by_number(2).unwrap().name, "Helium");
        assert_eq!(p.by_number(3).unwrap().name, "Lithium");

        assert_eq!(p.by_name("Hydrogen").unwrap().symbol, "H");
        assert_eq!(p.by_name("Helium").unwrap().symbol, "He");
        assert_eq!(p.by_name("Lithium").unwrap().symbol, "Li");

        assert_eq!(p.by_symbol("H").unwrap().number.get(), 1);
        assert_eq!(p.by_symbol("He").unwrap().number.get(), 2);
        assert_eq!(p.by_symbol("Li").unwrap().number.get(), 3);
    }
}
