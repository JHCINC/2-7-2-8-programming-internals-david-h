use std::{iter::Peekable, num::NonZeroUsize};

use anyhow::bail;

use crate::periodic_table::ElementNumber;

use super::{ComponentType, Equation, EquationConstituent};

#[derive(Clone, Debug)]
pub enum Token {
    Coefficient(NonZeroUsize),
    Component(ComponentType),
    Plus,
    Arrow,
}

/// Parse an entire chemical equation.
pub fn parse_equation(i: impl Iterator<Item = Token>) -> anyhow::Result<Equation> {
    let mut i = i.peekable();

    let mut equation = Equation::default();
    // grab the first reactant
    equation.reactants.push(parse_constituent(&mut i)?);

    let mut targeting_products = false;

    while let Some(token) = i.next() {
        let mut should_parse_constituent = false;
        match token {
            // if we find a plus
            Token::Plus => {
                should_parse_constituent = true; // parse a constituent.
            }
            // if we find an arrow
            Token::Arrow => {
                // bail if we have already found one
                if targeting_products {
                    bail!("Multiple equals in equation");
                }
                // otherwise note that we are
                // now targeting the products.
                targeting_products = true;

                // and parse another constituent.
                should_parse_constituent = true;
            }
            v => bail!("Unexpected token in input: {v:?}"),
        }

        if should_parse_constituent {
            // find our target
            let target = if targeting_products {
                &mut equation.products
            } else {
                &mut equation.reactants
            };

            // and add the parsed constituent.
            target.push(parse_constituent(&mut i)?);
        }
    }
    Ok(equation)
}

/// Parse one constituent of an equation.
///
/// Example:
/// Given an equation like H2 + O2 -> H2O,
/// H2, O2 and H2O are each the constituents.
pub fn parse_constituent(
    i: &mut Peekable<impl Iterator<Item = Token>>,
) -> anyhow::Result<EquationConstituent> {
    // look for the coefficient first. If one
    // is not present assume a coefficient of one.
    let coefficient = match i.peek().cloned() {
        Some(Token::Coefficient(c)) => {
            i.next(); // advance the iterator
            c
        }
        None => bail!("End of stream"),
        _ => NonZeroUsize::new(1).unwrap(), // default coefficient
    };

    let mut components = vec![];

    // parse as many components as there are to this constituent
    while let Some(Token::Component(c)) = i.peek().cloned() {
        i.next(); // advance
        components.push(c);
    }

    Ok(EquationConstituent {
        coefficient,
        components,
    })
}

#[cfg(test)]
mod tests {
    use std::num::{NonZeroU32, NonZeroUsize};

    use super::{parse_equation, Token};

    fn nzus(u: usize) -> NonZeroUsize {
        NonZeroUsize::new(u).unwrap()
    }

    fn nz32(u: u32) -> NonZeroU32 {
        NonZeroU32::new(u).unwrap()
    }

    // #[test]
    // fn basic_equation_parsing() {
    //     // tokens for water balanced eqn (2H2 + O2 -> 2H2O)
    //     let tokens = [
    //         Token::Coefficient(nzus(2)), // 2
    //         Token::Element {
    //             subscript: nzus(2),
    //             element: nz32(1),
    //         }, // H2
    //         Token::Plus,                 // +
    //         Token::Element {
    //             subscript: nzus(2),
    //             element: nz32(8),
    //         }, // O2
    //         Token::Arrow,                // ->
    //         Token::Coefficient(nzus(2)), // 2
    //         Token::Element {
    //             subscript: nzus(2),
    //             element: nz32(1),
    //         }, // H2
    //         Token::Element {
    //             subscript: nzus(1),
    //             element: nz32(8),
    //         }, // O
    //     ];

    //     let parsed = parse_equation(tokens.into_iter()).unwrap();

    //     assert_eq!(parsed.products.len(), 1);
    //     assert_eq!(parsed.reactants.len(), 2);

    //     // check reactants
    //     assert_eq!(parsed.reactants[0].components[0].element, nz32(1)); // hydrogen
    //     assert_eq!(parsed.reactants[1].components[0].element, nz32(8)); // oxygen

    //     // check products
    //     assert_eq!(parsed.products[0].components[0].element, nz32(1)); // hydrogen
    //     assert_eq!(parsed.products[0].components[1].element, nz32(8)); // oxygen
    // }
}
