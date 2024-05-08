use std::ops::{AddAssign, MulAssign};

use nalgebra::{DMatrixView, DMatrixViewMut, MatrixView1};

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
/// Currently not - using a library
pub fn gaussian_elimination(m: &mut DMatrixViewMut<f64>) {
    let mut values = vec![];
    for row in m.row_iter() {
        let mut v = vec![];
        for value in row.iter() {
            v.push(*value);
        }
        values.push(v);
    }

    gauss_jordan_elimination::gauss_jordan_elimination_generic(&mut values);

    for (row_idx, mut row) in m.row_iter_mut().enumerate() {
        for (col_idx, val) in row.iter_mut().enumerate() {
            *val = values[row_idx][col_idx];
        }
    }
}

fn gaussian_elimination_phase_one(m: &mut DMatrixViewMut<f64>, skip: usize) {
    let leftmost_nz = 'lnz: {
        for (col_idx, col) in m.column_iter().enumerate() {
            for (row_idx, row) in col.iter().enumerate().skip(skip) {
                println!("val: {row}");
                if *row != 0.0 {
                    println!("break");
                    break 'lnz (col_idx, row_idx, *row);
                }
            }
        }
        return;
    };

    swap(m, 0, leftmost_nz.1);
    mult(m, 0, 1.0 / leftmost_nz.2);

    for (row, val) in m
        .column(0)
        .clone_owned()
        .into_iter()
        .enumerate()
        .skip(skip + 1)
    {
        add(m, row, 0, -val);
    }

    if m.shape().1 == 1 {
        return;
    }

    gaussian_elimination_phase_one(&mut m.view_range_mut(1.., ..), skip + 1);
}

#[cfg(test)]
mod tests {
    use nalgebra::{dmatrix, DMatrix};

    use super::gaussian_elimination;

    #[test]
    fn gaussian_elimination_test_case() {
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

        let mut m = dmatrix![
            2., 0., 2.;
            0., 2., 1.;
        ];
        // assert_eq!(gaussian_elimination(&mut m.view_range_mut(.., ..)), [1.0, 0.5]);
    }
}
