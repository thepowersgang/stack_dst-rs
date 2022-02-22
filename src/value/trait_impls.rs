use core::future;
use core::pin;
use core::task;

macro_rules! d {
    ( $t:path; $($body:tt)* ) => {
        impl<D: ::DataBuf, T: ?Sized> $t for super::ValueA<T, D>
        where
            T: $t,
        {
            $( $body )*
        }
    }
}

d! { future::Future;
    type Output = T::Output;
    fn poll(self: pin::Pin<&mut Self>, cx: &mut task::Context) -> task::Poll<Self::Output> {
        unsafe { pin::Pin::new_unchecked(&mut **self.get_unchecked_mut()).poll(cx) }
    }
}
d! { ::core::iter::Iterator;
    type Item = T::Item;
    fn next(&mut self) -> Option<Self::Item> {
        (**self).next()
    }
}
d! { ::core::iter::ExactSizeIterator;
}

macro_rules! impl_fmt {
    ( $( $t:ident )* ) => {
        $(
            d!{ ::core::fmt::$t;
                fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                    (**self).fmt(f)
                }
            }
        )*
    }
}
impl_fmt!{
    Display Debug UpperHex LowerHex
}
