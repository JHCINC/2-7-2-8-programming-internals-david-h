use std::ops::{MulAssign, AddAssign};

use nalgebra::{DMatrixViewMut, DMatrixView, MatrixView1};


// not a function because of weird nalgebra types - should not be a macro
macro_rules! leading_nz {
    ($matrix:expr) => {
        'outer: {
            for (index, val) in $matrix.iter().enumerate() {
                if *val != 0.0 {
                    break 'outer Some(index);
                }
            }
            None
        }
    };
}

fn swap(m: &mut DMatrixViewMut<f64>, a: usize, b: usize) {
    m.swap_rows(a, b)
}

fn mult(m: &mut DMatrixViewMut<f64>, a: usize, scalar: f64) {
    m.row_mut(a).mul_assign(scalar);
}

fn add(m: &mut DMatrixViewMut<f64>, a: usize, b: usize, scalar: f64) {
    let b = m.row(b).clone_owned();
    m.row_mut(a).add_assign(b * scalar);
}


/// Built from:
/// https://ximera.osu.edu/linearalgebra/textbook/rowReduction/algorithm
fn gaussian_elimination(m: &mut DMatrixViewMut<f64>) {

    gaussian_elimination_phase_one(m, 0); // steps 1 to 5




    // let (ncols, nrows) = m.shape();

    // // go over all rows from bottom
    // for i in (0..nrows).rev() {


    //     // if the row has a leading one
    //     if let Some(nz) = leading_nz!(&m.row(i)) {

    //         for (row, val) in m.column(nz).clone_owned().into_iter().enumerate().take(i) {
    //             add(m, row, i, -val);
    //         }

    //     }
    // }

    
}

fn gaussian_elimination_phase_one(m: &mut DMatrixViewMut<f64>, skip: usize) {


    let leftmost_nz = 'lnz: { for (col_idx, col) in m.column_iter().enumerate() {
            for (row_idx, row) in col.iter().enumerate().skip(skip) {
                println !("val: {row}");
                if *row != 0.0 {
                    println!("break");
                    break 'lnz (col_idx, row_idx, *row);
                }
            }
        };
        return
    };

    swap(m, 0, leftmost_nz.1);
    mult(m, 0, 1.0 / leftmost_nz.2);

    for (row, val) in m.column(0).clone_owned().into_iter().enumerate().skip(skip + 1) {
        add(m, row, 0, -val);
    }

    if m.shape().1 == 1 {
        return;
    }

    gaussian_elimination_phase_one(&mut m.view_range_mut(1.., ..), skip + 1);

}

#[cfg(test)]
mod tests {
    use nalgebra::{DMatrix, dmatrix};

    use super::gaussian_elimination;

    
    fn test_matrices(mut original: DMatrix<f64>, expected: DMatrix<f64>) {

        gaussian_elimination(&mut original.view_range_mut(.., ..));

        if original != expected {
            panic!("assertion `original == expected` failed: {} {}", original, expected);
        }
    }
    
    
    
    #[test]
    fn gaussian_elimination_test_case() {
        test_matrices(DMatrix::identity(3, 3), DMatrix::identity(3, 3));  
        
        // test_matrices(
        //     dmatrix![
        //         1.0, 0.0, 0.0;
        //         1.0, 1.0, 0.0;
        //         0.0, 0.0, 1.0;
        //     ],
        //     DMatrix::identity(3, 3)
        // );

        // test_matrices(
        //     dmatrix![
        //         1.0, 0.0, 2.0;
        //         1.0, 1.0, 0.0;
        //         0.0, 0.0, 1.0;
        //     ],
        //     DMatrix::identity(3, 3)
        // );

        test_matrices(
            dmatrix![
                9., 2., 1.;
                3., 2., 5.;
                6., 7., 7.;
            ],
            DMatrix::identity(3, 3)
        );
    }
}