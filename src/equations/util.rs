use nalgebra::DMatrixViewMut;

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
