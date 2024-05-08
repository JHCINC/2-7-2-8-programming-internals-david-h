use std::{collections::HashMap, num::NonZeroUsize};

use anyhow::bail;
use bimap::BiMap;
use nalgebra::DMatrix;

use super::Equation;


pub fn balance_equation(eq: &mut Equation) -> anyhow::Result<()> {
    if eq.products.is_empty() {
        return Ok(()); // no products - can't balance.
    }

    let mut matrix = create_matrix(eq)?;
    
    super::util::gaussian_elimination(&mut matrix.view_range_mut(.., ..));
    
    let mut solutions = matrix.row(matrix.nrows() - 1).clone_owned().data.as_mut_slice()[1..].to_vec();
    
    solutions.iter_mut().for_each(|v| *v = v.abs());

    // supremely naive.
    
    // if contains non-integers
    if !solutions.iter().all(|v| v.fract() == 0.0) {
        // try scalars up to 100

        let mut solutions_clone = solutions.clone();
        for scalar in 2..100 {
            
            for value in solutions_clone.iter_mut() {
                *value *= scalar as f64;
            }

            if solutions_clone.iter().all(|v| v.fract() == 0.0) {
                solutions = solutions_clone;
                solutions.push(scalar as f64);
                break;
            } else {
                solutions_clone = solutions.clone();
            }

        }
    }
    for (v, value) in eq.reactants.iter_mut().chain(eq.products.iter_mut()).zip(solutions.into_iter()) {
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
    let mut matrix = DMatrix::<f64>::zeros(element_order.len(), eq.num_products() + eq.num_reactants());


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

    use crate::{equations::{solve::create_matrix, Equation, EquationConstituent}, periodic_table::{TablePrintable, PeriodicTable}};

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
                2., 0., 2.;
                0., 2., 1.;
            ]
        );

        let mut eq = eq;
        let p = PeriodicTable::from_json(std::fs::File::open("./PeriodicTableJSON.json").unwrap()).unwrap();
        super::balance_equation(&mut eq).unwrap();
        panic!("{}", eq.to_string(&p).unwrap());
    }
}
