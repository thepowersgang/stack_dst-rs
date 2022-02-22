macro_rules! d {
    ( $t:path; $($body:tt)* ) => {
        impl<D: ::DataBuf, T: ?Sized> $t for super::StackA<T, D>
        where
            T: $t,
        {
            $( $body )*
        }
    }
}

d! { ::core::fmt::Debug;
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        f.write_str("[")?;
        for v in self.iter() {
            v.fmt(f)?;
            f.write_str(",")?;
        }
        f.write_str("]")?;
        Ok( () )
    }
}
