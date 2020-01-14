
use core::pin;
use core::task;
use core::future;
use core::fmt;

/// Future if the inner impls Future
impl<D: ::DataBuf,T: ?Sized> future::Future for super::ValueA<T, D>
where
	T: future::Future
{
	type Output = T::Output;
	fn poll(self: pin::Pin<&mut Self>, cx: &mut task::Context) -> task::Poll<Self::Output> {
		unsafe {
			pin::Pin::new_unchecked(&mut **self.get_unchecked_mut()).poll(cx)
		}
	}
}

impl<D: ::DataBuf,T: ?Sized> fmt::Display for super::ValueA<T, D>
where
	T: fmt::Display
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		(**self).fmt(f)
	}
}

impl<D: ::DataBuf,T: ?Sized> fmt::Debug for super::ValueA<T, D>
where
	T: fmt::Debug
{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		(**self).fmt(f)
	}
}
