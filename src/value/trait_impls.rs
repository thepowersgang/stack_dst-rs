use core::future;
use core::pin;
use core::task;

macro_rules! d {
    ( $t:path; $($body:tt)* ) => {
        impl<D: ::DataBuf, T: ?Sized> $t for super::Value<T, D>
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
    // NOTE: Only a few methods can be directly passed through
    // Namely, those that don't use `self` by value and don't use generics

    // Included because it's actually useful API information
    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }

    // Included because it can be
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        (**self).nth(n)
    }
}
d! { ::core::iter::DoubleEndedIterator;
    fn next_back(&mut self) -> Option<Self::Item> {
        (**self).next_back()
    }

    // Included because it can be
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        (**self).nth_back(n)
    }
}
d! { ::core::iter::ExactSizeIterator;
    fn len(&self) -> usize { (**self).len() }

    // Unstable
    //fn is_empty(&self) -> bool { (**self).is_empty() }
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
impl_fmt! {
    Display Debug UpperHex LowerHex
}
