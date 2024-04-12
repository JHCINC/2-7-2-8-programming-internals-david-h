use std::{collections::HashMap, num::NonZeroUsize};

use crate::periodic_table::ElementNumber;

mod parse;
mod solve;
mod util;

pub struct Component {
    pub element: ElementNumber,
    pub subscript: NonZeroUsize,
}
pub struct EquationConstituent {
    pub coefficient: NonZeroUsize,
    pub components: Vec<Component>,
}

impl EquationConstituent {

    pub fn new(coefficient: usize, components: &[(ElementNumber, usize)]) -> Self {
        let mut components_store = vec![];
        for (element, count) in components {
            components_store.push(Component {
                element: *element,
                subscript: NonZeroUsize::new(*count).unwrap()
            });
        } 
        Self {
            coefficient: NonZeroUsize::new(coefficient).unwrap(),
            components: components_store
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

#[derive(Default)]
pub struct Equation {
    reactants: Vec<EquationConstituent>,
    products: Vec<EquationConstituent>,
}

impl Equation {

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
    use std::num::{NonZeroUsize, NonZeroU32};

    use super::{EquationConstituent, Component};

    #[test]
    fn constituent_elements() {
        let mut con = EquationConstituent {
            coefficient: NonZeroUsize::new(3).unwrap(),
            components: vec![
                Component { element: NonZeroU32::new(1).unwrap(), subscript: NonZeroUsize::new(2).unwrap() }
            ],
        };

        let vals = con.elements().collect::<Vec<_>>();
        assert_eq!(vals.len(), 1); // one element
        assert_eq!(vals[0].0.get(), 1); // element is hydrogen
        assert_eq!(vals[0].1, 6); // six atoms
    }
}