
use core::pin::Pin;
use core::task::{Context,Poll};

impl<D: ::DataBuf,T> core::future::Future for super::ValueA<dyn core::future::Future<Output=T>, D>
{
	type Output = T;
	fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
		unsafe {
			Pin::new_unchecked(&mut **self.get_unchecked_mut()).poll(cx)
		}
	}
}
