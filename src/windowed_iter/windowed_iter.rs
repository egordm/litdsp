use litcontainers::*;

pub trait WindowedIter<'a, T, W, H>: Sized
	where T: Scalar, W: Dim, H: Dim
{
	type WC: Dim;
	type WCS: Dim;
	type WR: Dim;
	type WRS: Dim;

	fn next_window<'at: 'b, 'b>(&'at mut self) -> Option<Slice<'b, T, Self::WR, Self::WRS, Self::WC, Self::WCS>>;

	fn window_size(&self) -> usize { self.window_dim().value() }

	fn window_dim(&self) -> W;

	fn hop_size(&self) -> usize { self.hop_dim().value() }

	fn hop_dim(&self) -> H;

	fn window_count(&self) -> usize;
}

pub trait WindowedIterMut<'a, T, W, H>: WindowedIter<'a, T, W, H>
	where T: Scalar,  W: Dim, H: Dim
{
	fn next_window_mut<'at: 'b, 'b>(&'at mut self) -> Option<SliceMut<'b, T, Self::WR, Self::WRS, Self::WC, Self::WCS>>;
}