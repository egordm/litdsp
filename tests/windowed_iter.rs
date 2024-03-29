use litcontainers::*;
use litdsp::windowed_iter::*;

#[test]
fn windowed_col_iter() {
	let s = ContainerRM::from_vec(Size::new(U1, D!(4)), &[1., 2., 3., 4.]);
	let slice = s.slice_rows(0);

	let mut iter = WindowedColIter::new_padded(&slice, U3, U2, 1, 2);
	let mut ws = Vec::new();
	while let Some(w) = iter.next_window() {
		ws.push(w.as_slice().to_vec())
	}

	assert_eq!(iter.window_count(), 3);
	assert_eq!(ws, &[vec![0., 1., 2.], vec![2., 3., 4.], vec![4., 0., 0.]]);

	let s = ContainerRM::from_vec(Size::new(U1, D!(40)), &[0; 40]);
	let slice = s.slice_rows(0);
	let iter = WindowedColIter::new_padded(&slice, U12, U6, 0, 0);
	assert_eq!(iter.window_count(), 5);
}

#[test]
fn windowed_row_iter() {
	let s = ContainerCM::from_vec(Size::new(D!(4), U1), &[1., 2., 3., 4.]);
	let slice = s.slice_cols(0);

	let mut iter = WindowedRowIter::new_padded(&slice, U3, U2, 1, 2);
	let mut ws = Vec::new();
	while let Some(w) = iter.next_window() {
		ws.push(w.as_slice().to_vec())
	}

	assert_eq!(iter.window_count(), 3);
	assert_eq!(ws, &[&[0., 1., 2.], &[2., 3., 4.], &[4., 0., 0.]]);
}