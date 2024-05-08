use std::ops::{AddAssign, MulAssign};

use nalgebra::{DMatrixView, DMatrixViewMut, MatrixView1};

/// from https://github.com/TheAlgorithms/Rust/blob/master/src/math/gaussian_elimination.rs
mod stolen_impl {

    pub fn gaussian_elimination(matrix: &mut [Vec<f64>]) -> Vec<f64> {
        let size = matrix.len();
        assert_eq!(size, matrix[0].len() - 1);

        for i in 0..size - 1 {
            for j in i..size - 1 {
                echelon(matrix, i, j);
            }
        }

        for i in (1..size).rev() {
            eliminate(matrix, i);
        }

        // Disable cargo clippy warnings about needless range loops.
        // Checking the diagonal like this is simpler than any alternative.
        #[allow(clippy::needless_range_loop)]
        for i in 0..size {
            if matrix[i][i] == 0f64 {
                println!("Infinitely many solutions");
            }
        }

        let mut result: Vec<f64> = vec![0f64; size];
        for i in 0..size {
            result[i] = matrix[i][size] / matrix[i][i];
        }
        result
    }

    fn echelon(matrix: &mut [Vec<f64>], i: usize, j: usize) {
        let size = matrix.len();
        if matrix[i][i] == 0f64 {
        } else {
            let factor = matrix[j + 1][i] / matrix[i][i];
            (i..size + 1).for_each(|k| {
                matrix[j + 1][k] -= factor * matrix[i][k];
            });
        }
    }

    fn eliminate(matrix: &mut [Vec<f64>], i: usize) {
        let size = matrix.len();
        if matrix[i][i] == 0f64 {
        } else {
            for j in (1..i + 1).rev() {
                let factor = matrix[j - 1][i] / matrix[i][i];
                for k in (0..size + 1).rev() {
                    matrix[j - 1][k] -= factor * matrix[i][k];
                }
            }
        }
    }
}

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
/// Currently is not - stolen implementation from https://github.com/TheAlgorithms/Rust/blob/master/src/math/gaussian_elimination.rs for testing
pub fn gaussian_elimination(m: &mut DMatrixViewMut<f64>) -> Vec<f64> {
    println!("val: {}", m);
    let mut values = vec![];
    for row in m.row_iter() {
        let mut v = vec![];
        for value in row.iter() {
            v.push(*value);
        }
        values.push(v);
    }

    let v = stolen_impl::gaussian_elimination(&mut values);
    for (row_idx, mut row) in m.row_iter_mut().enumerate() {
        for (col_idx, val) in row.iter_mut().enumerate() {
            *val = values[row_idx][col_idx];
        }
    }

    v
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
        assert_eq!(gaussian_elimination(&mut m.view_range_mut(.., ..)), [1.0, 0.5]);
    }
}
