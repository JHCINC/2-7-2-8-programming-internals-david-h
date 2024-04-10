use std::collections::HashMap;

use anyhow::bail;
use bimap::BiMap;
use nalgebra::DMatrix;

use super::Equation;

fn create_matrix(eq: &Equation) -> anyhow::Result<DMatrix<usize>> {
    let products = eq.total_product_elements();

    let mut element_order = BiMap::new();
    for (index, product) in products.iter().enumerate() {
        element_order.insert(product.0, index);
    }

    // create a matrix with as many rows
    // as there are element products and
    // as many columns as there are reactants
    // plus one, accounting for the product.
    let mut matrix = DMatrix::<usize>::zeros(products.len(), eq.num_reactants() + 1);

    let mut last_column = matrix.column_mut(products.len());

    // fill the last column with the products
    for (index, value) in last_column.iter_mut().enumerate() {
        let element_for_slot = element_order.get_by_right(&index).unwrap();
        *value = products[element_for_slot]
    }

    // fill each row with the number of
    // atoms in each constituent
    for (index, constituent) in eq.reactants().iter().enumerate() {
        let mut row = matrix.row_mut(index);

        for (element, count) in constituent.elements() {
            let Some(index) = element_order.get_by_left(&element).copied() else {
                bail!("Unused constituent on LHS")
            };

            *row.get_mut(index).expect("Matrix should be correct size") = count;
        }
    }

    Ok(matrix)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU32;

    use nalgebra::matrix;

    use crate::equations::{solve::create_matrix, Equation, EquationConstituent};

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
                0, 2, 1;
                2, 0, 2;
            ]
        )
    }
}
