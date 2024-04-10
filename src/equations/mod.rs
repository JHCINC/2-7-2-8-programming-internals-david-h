use std::num::NonZeroUsize;

use crate::periodic_table::ElementNumber;

mod parse;


pub struct Component {
    pub element: ElementNumber,
    pub subscript: NonZeroUsize
}
pub struct EquationConstituent {
    pub coefficient: NonZeroUsize,
    pub components: Vec<Component>
}

#[derive(Default)]
pub struct Equation {
    pub reactants: Vec<EquationConstituent>,
    pub products: Vec<EquationConstituent>
}