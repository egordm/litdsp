use litcontainers::*;
use crate::*;
use crate::windowed_iter::{WindowedColIter, WindowedIter, WindowedIterMut};
use crate::algorithms::{STFTF64Feed, ISTFTF64Feed};
use rayon::prelude::*;

pub fn calculate_pv<S, W>(s: &S, sr: f64, alpha: f32, window_dim: W) -> RowVec<f64, Dynamic>
	where S: RowVecStorage<f64>, W: Dim, W: Dim + DimDiv<U2>, <W as DimDiv<U2>>::Output: DimAdd<U1>
{
	let win_size = window_dim.value();
	let padding = win_size / 2;
	let syn_hop_size = win_size / 4;
	let in_hop_size  = (syn_hop_size as f32 / alpha).round() as usize;

	let window = window::sinw(window_dim, 1.);
	let window_inv = &window / ((win_size as f64 / syn_hop_size as f64) / 2.).sqrt();

	let omega = RowVec::regspace(Size::new(U1, D!(win_size / 2 + 1)), RowAxis, 0.) * (constants::PI2 / win_size as f64);

	let mut window_iter = WindowedColIter::new_padded(s, window.col_dim(), D!(in_hop_size), padding, padding);
	let out_size = (window_iter.window_count() - 1) * syn_hop_size + win_size;
	let mut output = RowVec::zeros(Size::new(U1, D!(out_size)));

	let mut stft_plan = STFTF64Feed::new(window_dim.clone());
	let mut istft_plan = ISTFTF64Feed::new(window_dim.clone());

	let mut phase_current = RowVec::zeros(Size::new(U1, stft_plan.out_dim()));
	let mut phase_syn = RowVec::zeros(Size::new(U1, stft_plan.out_dim()));
	let mut syn_data = RowVec::zeros(Size::new(U1, stft_plan.out_dim()));

	let mut S = RowVec::zeros(Size::new(U1, stft_plan.out_dim()));
	let mut sv = RowVec::zeros(Size::new(U1, istft_plan.out_dim()));


	// Is it possible to parlallelize? like split in half & slow down both?
	// Maybe calc full stft -> go through keeping phase (serial) -> calculate phase syn which can be patallel

	let mut cursor = 0;
	while let Some(mut w) = window_iter.next_window_mut() {
		w *= &window;
		stft_plan.next(&w, &mut S);

		for (i, s_i) in S.as_iter().enumerate() {
			let phase_previous = phase_current[i];
			phase_current[i] = f64::atan2(s_i.im, s_i.re);

			if cursor != 0 {
				let delta = (phase_current[i] - phase_previous) - omega[i] * in_hop_size as f64;
				let delta = delta - constants::PI2 * (delta / constants::PI2).round(); // Put delta in 2PI range

				let y_unwrap = omega[i] + delta / in_hop_size as f64;
				phase_syn[i] += y_unwrap * syn_hop_size as f64;
			} else {
				phase_syn[i] = phase_current[i];
			}

			let mag = s_i.norm();
			syn_data[i] = c64::new(mag * phase_syn[i].cos(), mag * phase_syn[i].sin());
		}

		istft_plan.next(&syn_data, &mut sv);
		sv *= &window_inv;

		let mut sl = output.slice_cols_mut(SizedRange::new((cursor * syn_hop_size) as usize, w.col_dim()));
		sl += &sv;
		cursor += 1;
	}

	output
}

/*
pub fn calculate_pv_test<S, W>(s: &S, sr: f64, alpha: f32, window_dim: W) -> RowVec<f64, Dynamic>
	where S: RowVecStorage<f64>, W: Dim, W: Dim + DimDiv<U2>, <W as DimDiv<U2>>::Output: DimAdd<U1>
{
	let win_size = window_dim.value();
	let padding = win_size / 2;
	let syn_hop_size = win_size / 4;
	let in_hop_size  = (syn_hop_size as f32 / alpha).round() as usize;

	let window = window::sinw(window_dim, 1.);
	let window_inv = &window / ((win_size as f64 / syn_hop_size as f64) / 2.).sqrt();

	let omega = RowVec::regspace(Size::new(U1, D!(win_size / 2 + 1)), RowAxis, 0.) * (constants::PI2 / win_size as f64);

	let (mut S, ssr) = stft::calculate_stft(s, &window, D!(in_hop_size), true, sr);

	let out_size = (S.cols() - 1) * syn_hop_size + win_size;
	let mut output = RowVec::zeros(Size::new(U1, D!(out_size)));

	let stft_size = S.row_dim();

	let mut syn_data = ContainerCM::zeros(S.size());

	let chunk_size = S.cols() / 8;
	(0..S.cols()).into_par_iter().zip(syn_data.as_col_slice_iter_mut()).chunks(chunk_size).for_each(|is, sd| {
		let mut phase_previous = RowVec::zeros(Size::new(U1, stft_size));
		let mut phase_current = RowVec::zeros(Size::new(U1, stft_size));
		let mut phase_syn = RowVec::zeros(Size::new(U1, stft_size));

		for i in is {
			let syn_data = syn_data.slice_cols_mut(i);
			let s_slice = S.slice_cols(i);
			phase_previous.copy_from(&phase_current);
			phase_current.mapv_inplace_zip(s_slice.as_col_iter(), |_, s| f64::atan2(s.im, s.re));

			if i == 0 {
				phase_syn.copy_from(&phase_current);
			} else {
				for j in 0..phase_syn.cols() {
					let delta = (phase_current[j] - phase_previous[j]) - omega[j] * in_hop_size as f64;
					let delta = delta - constants::PI2 * (delta / constants::PI2).round(); // Put delta in 2PI range
					let y_unwrap = omega[j] + delta / in_hop_size as f64;
					phase_syn[j] += y_unwrap * syn_hop_size as f64;
				}
			}

			for j in 0..phase_syn.cols() {
				let mag = s_slice[j].norm();
				syn_data[j] = c64::new(mag * phase_syn[j].cos(), mag * phase_syn[j].sin());
			}
		}

		let u = 0;
	});

	output
}*/
