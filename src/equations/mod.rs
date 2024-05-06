use std::{collections::HashMap, num::NonZeroUsize};

use crate::{
    periodic_table::{ElementNumber, PeriodicTable, TablePrintable},
    tui::subscript_util,
};

use self::parse::{parse_equation, Token};

pub mod parse;
mod solve;
mod util;

#[derive(Debug, Clone)]
pub struct Component {
    pub element: ElementNumber,
    pub subscript: NonZeroUsize,
}
impl TablePrintable for Component {
    fn fmt(
        &self,
        t: &crate::periodic_table::PeriodicTable,
        f: &mut impl std::fmt::Write,
    ) -> std::fmt::Result {
        let name = &t.by_number(self.element.get()).unwrap().symbol;
        write!(f, "{name}")?;

        let subscript = self.subscript.get();
        if subscript > 1 {
            write!(f, "{}", subscript_util(subscript as u32))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct EquationConstituent {
    pub coefficient: NonZeroUsize,
    pub components: Vec<Component>,
}

impl TablePrintable for EquationConstituent {
    fn fmt(
        &self,
        t: &crate::periodic_table::PeriodicTable,
        f: &mut impl std::fmt::Write,
    ) -> std::fmt::Result {
        let coefficient = self.coefficient.get();
        if coefficient > 1 {
            write!(f, "{}", coefficient)?;
        }
        for c in &self.components {
            TablePrintable::fmt(c, t, f)?;
        }
        Ok(())
    }
}

impl EquationConstituent {
    pub fn new(coefficient: usize, components: &[(ElementNumber, usize)]) -> Self {
        let mut components_store = vec![];
        for (element, count) in components {
            components_store.push(Component {
                element: *element,
                subscript: NonZeroUsize::new(*count).unwrap(),
            });
        }
        Self {
            coefficient: NonZeroUsize::new(coefficient).unwrap(),
            components: components_store,
        }
    }

    /// Returns the total number of element
    /// atoms present, accounting for the coefficient.
    pub fn elements(&self) -> impl Iterator<Item = (ElementNumber, usize)> + '_ {
        self.components.iter().map(|v| {
            (
                v.element,
                v.subscript
                    .get()
                    .checked_mul(self.coefficient.get())
                    .expect("Overflow during constituent calculation"),
            )
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct Equation {
    reactants: Vec<EquationConstituent>,
    products: Vec<EquationConstituent>,
}
impl TablePrintable for Equation {
    fn fmt(
        &self,
        t: &crate::periodic_table::PeriodicTable,
        f: &mut impl std::fmt::Write,
    ) -> std::fmt::Result {
        let num_reactants = self.reactants.len();
        for (idx, c) in self.reactants.iter().enumerate() {
            TablePrintable::fmt(c, t, f)?;
            if idx != num_reactants - 1 {
                write!(f, " + ")?;
            }
        }
        if !self.products.is_empty() {
            write!(f, " = ")?;
            for product in &self.products {
                TablePrintable::fmt(product, t, f)?;
            }
        }
        Ok(())
    }
}

impl Equation {
    pub fn balanced(&mut self) -> anyhow::Result<Self> {
        let mut new = self.clone();
        solve::balance_equation(&mut new)?;
        Ok(new)
    }

    pub fn to_string(&self, p: &PeriodicTable) -> anyhow::Result<String> {
        let mut buf = String::new();
        TablePrintable::fmt(self, p, &mut buf)?;
        Ok(buf)
    }

    pub fn reactants(&self) -> &[EquationConstituent] {
        &self.reactants
    }

    pub fn num_products(&self) -> usize {
        self.products.len()
    }

    pub fn num_reactants(&self) -> usize {
        self.reactants.len()
    }

    /// Returns all of the elements present
    /// in the product with duplicates
    /// coalesced.
    pub fn total_product_elements(&self) -> HashMap<ElementNumber, usize> {
        let mut elements = HashMap::new();

        // iterate over all elements in the products
        for (element, count) in self.products.iter().map(|v| v.elements()).flatten() {
            *elements.entry(element).or_default() += count; // handle duplicates with a map
        }

        elements
    }
}

#[cfg(test)]
mod tests {
    use std::num::{NonZeroU32, NonZeroUsize};

    use super::{Component, EquationConstituent};

    #[test]
    fn constituent_elements() {
        let mut con = EquationConstituent {
            coefficient: NonZeroUsize::new(3).unwrap(),
            components: vec![Component {
                element: NonZeroU32::new(1).unwrap(),
                subscript: NonZeroUsize::new(2).unwrap(),
            }],
        };

        let vals = con.elements().collect::<Vec<_>>();
        assert_eq!(vals.len(), 1); // one element
        assert_eq!(vals[0].0.get(), 1); // element is hydrogen
        assert_eq!(vals[0].1, 6); // six atoms
    }
}
