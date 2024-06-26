use std::num::NonZeroUsize;

use anyhow::bail;
use bimap::BiMap;
use nalgebra::DMatrix;

use super::Equation;

pub fn balance_equation(eq: &mut Equation) -> anyhow::Result<()> {
    if eq.products.is_empty() {
        return Ok(()); // no products - can't balance.
    }

    let mut matrix = create_matrix(eq)?;

    // KOH + H`3PO`4 = K`3PO`4 + H`2O
    // CaCl`2 + Na`3PO`4 = Ca`3(PO`4)`2 + NaCl

    // test eqns
    //panic!("Matrix: {matrix}");

    super::util::gaussian_elimination(&mut matrix.view_range_mut(.., ..));

    let mut solutions = matrix
        .column(matrix.ncols() - 1)
        .clone_owned()
        .data
        .as_slice()
        .to_vec();

    solutions.iter_mut().for_each(|v| *v = v.abs());

    // supremely naive and awful. should be done entirely with
    // fractions, not floating-point numbers. too lazy at this
    // point to architect a solution like that. certainly a
    // project for the future :^)

    // if contains non-integers

    let mut scalar_val = 0.0;
    if !solutions.iter().all(|v| v.fract() == 0.0) {
        // try scalars up to 100 - very naive

        let mut solutions_clone = solutions.clone();
        let mut found = false;
        for scalar in 2..100 {
            for value in solutions_clone.iter_mut() {
                *value *= scalar as f64;
                // 1/3 + 1/3 + 1/3 should = 1, not 0.9999
                // again a problem with using floats
                if (value.round() - *value).abs() < 0.0001 {
                    *value = value.round()
                }
            }

            if solutions_clone.iter().all(|v| v.fract() == 0.0) {
                solutions = solutions_clone;
                found = true;
                scalar_val = scalar as f64;
                break;
            } else {
                solutions_clone = solutions.clone();
            }
        }
        if !found {
            panic!("could not find coefficient")
        }
    }

    solutions.iter_mut().for_each(|v| {
        // 0-values replaced with scalar.
        // Why? ask a mathematician :^)
        // seems to work. This whole
        // matrix solving section
        // should really be
        // rewritten.
        if *v < f64::EPSILON {
            *v = scalar_val
        }
    });

    for (v, value) in eq
        .reactants
        .iter_mut()
        .chain(eq.products.iter_mut())
        .zip(solutions.into_iter().chain(std::iter::repeat(scalar_val)))
    {
        if let Some(nz) = NonZeroUsize::new(value as usize) {
            v.coefficient = nz;
        }
    }

    Ok(())
}

fn create_matrix(eq: &Equation) -> anyhow::Result<DMatrix<f64>> {
    let products = eq.total_product_elements();

    let mut element_order = BiMap::new();

    {
        let mut products_iter = products.iter().collect::<Vec<_>>();

        products_iter.sort_by(|a, b| a.0.cmp(b.0));

        for (index, product) in products_iter.into_iter().enumerate() {
            element_order.insert(product.0, index);
        }
    }

    // create a matrix with as many rows
    // as there are element constituents and
    // as many columns as there are components.
    let mut matrix =
        DMatrix::<f64>::zeros(element_order.len(), eq.num_products() + eq.num_reactants());

    for (index, constituent) in eq.reactants().iter().enumerate() {
        let mut col = matrix.column_mut(index);

        for (element, count) in constituent.elements() {
            let Some(index) = element_order.get_by_left(&element).copied() else {
                bail!("Unused constituent on LHS")
            };

            *col.get_mut(index).expect("Matrix should be correct size") = count as f64;
        }
    }

    for (index, constituent) in eq.products().iter().enumerate() {
        let mut col = matrix.column_mut(index + eq.num_reactants());

        for (element, count) in constituent.elements() {
            let Some(index) = element_order.get_by_left(&element).copied() else {
                bail!("Unused constituent on LHS")
            };

            *col.get_mut(index).expect("Matrix should be correct size") = -(count as f64);
        }
    }

    Ok(matrix)

    // // create a matrix with as many rows
    // // as there are element products and
    // // as many columns as there are reactants
    // // plus one, accounting for the product.
    // let mut matrix = DMatrix::<f64>::zeros(products.len(), eq.num_reactants() + 1);

    // let mut last_column = matrix.column_mut(products.len());

    // // fill the last column with the products
    // for (index, value) in last_column.iter_mut().enumerate() {
    //     let element_for_slot = element_order.get_by_right(&index).unwrap();
    //     println!("Elem in row {index} is {element_for_slot} val {}", products[element_for_slot]);
    //     *value = products[element_for_slot] as f64
    // }

    // // fill each row with the number of
    // // atoms in each constituent
    // for (index, constituent) in eq.reactants().iter().enumerate() {
    //     let mut row = matrix.column_mut(index);

    //     for (element, count) in constituent.elements() {

    //         let Some(index) = element_order.get_by_left(&element).copied() else {
    //             bail!("Unused constituent on LHS")
    //         };

    //         *row.get_mut(index).expect("Matrix should be correct size") = count as f64;
    //     }
    // }

    // Ok(matrix)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use nalgebra::matrix;

    use crate::{
        equations::{solve::create_matrix, Equation, EquationConstituent},
        periodic_table::PeriodicTable,
    };

    #[test]
    fn test_water() {
        let eq = Equation {
            reactants: vec![
                EquationConstituent::new(1, &[(NonZeroU32::new(1).unwrap(), 2)]),
                EquationConstituent::new(1, &[(NonZeroU32::new(8).unwrap(), 2)]),
            ],
            products: vec![EquationConstituent::new(
                1,
                &[
                    (NonZeroU32::new(1).unwrap(), 2),
                    (NonZeroU32::new(8).unwrap(), 1),
                ],
            )],
        }; // H2 + O2 -> H2O

        assert_eq!(
            create_matrix(&eq).unwrap(),
            matrix![
                2., 0., -2.;
                0., 2., -1.;
            ]
        );
    }
}
