use litcontainers::*;
use itertools::{Itertools, MinMaxResult};
use num_traits::Signed;
// TODO:  Make generic over storage

/// 1-D data interpolation
/// Interpolates to nearest neighbour from input values
/// # Arguments
/// * `axis_in` - Input axis
/// * `values_in` - Input values
/// * `axis_out` - Output axis
/// * `values_out` - Output values
pub fn interp1_nearest_cols<TA, TV, AIS, IS, AOS, OS, C>(axis_in: &AIS, values_in: &IS, axis_out: &AOS, values_out: &mut OS)
	where TA: Scalar + Signed, TV: Scalar, C: Dim,
	      AIS: ColVecStorage<TA>, IS: Storage<TV> + StorageSize<Rows=AIS::Rows, Cols=C>,
	      AOS: ColVecStorage<TA>, OS: StorageMut<TV> + StorageSize<Rows=AOS::Rows, Cols=C>
{
	assert!(axis_in.rows() == values_in.rows() && axis_out.rows() == axis_out.rows() && values_in.cols() == values_out.cols(), "Container dimensions are not valid");
	let (axis_in_min, axis_in_max) = match axis_in.as_col_iter().minmax() {
		MinMaxResult::NoElements => (TA::default(), TA::default()), // Should not be possible
		MinMaxResult::OneElement(v) => (*v, *v),
		MinMaxResult::MinMax(min, max) => (*min, *max),
	};

	let axis_size_in = axis_in.rows();
	let axis_size_out = axis_out.rows();

	let mut best_j = 0;

	for i in 0..axis_size_out {
		let mut best_err = TA::max_val();
		let axis_out_val = axis_out[i];

		if axis_out_val < axis_in_min {
			values_out.slice_rows_mut(i).copy_from(&values_in.slice_rows(0));
		} else if axis_out_val > axis_in_max {
			values_out.slice_rows_mut(i).copy_from(&values_in.slice_rows(axis_size_in - 1));
		} else {
			for j in best_j..axis_size_in {
				let err = (axis_in[j] - axis_out_val).abs();

				if err < best_err {
					best_err = err;
					best_j = j;
				} else {
					break;
				}
			}
			values_out.slice_rows_mut(i).copy_from(&values_in.slice_rows(best_j));
		}
	}
}