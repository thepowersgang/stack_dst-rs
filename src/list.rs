// See parent for docs
use ::core::{marker, mem, ops, ptr, iter};

mod impls;

// Implementation Notes
// -----
//
/// A First-In-First-Out queue of DSTs
///
/// ```
/// let mut queue = ::stack_dst::Fifo::<str, 8>::new();
/// queue.push_back_str("Hello");
/// queue.push_back_str("World");
/// assert_eq!(queue.pop_front().as_deref(), Some("Hello"));
/// ```
pub struct FifoA<T: ?Sized, D: ::DataBuf>
{
    _align: [u64; 0],
    _pd: marker::PhantomData<*const T>,
    read_pos: usize,
    write_pos: usize,
    data: D,
}
impl<T: ?Sized, D: ::DataBuf> FifoA<T,D>
{
    /// Construct a new (empty) list
    pub fn new() -> Self {
        FifoA {
            _align: [],
            _pd: marker::PhantomData,
            read_pos: 0,
            write_pos: 0,
            data: D::default(),
        }
    }

    fn meta_words() -> usize {
        D::round_to_words(mem::size_of::<&T>() - mem::size_of::<usize>())
    }
    fn space_words(&self) -> usize {
        self.data.as_ref().len() - self.write_pos
    }

    /// Push a value at the top of the stack
    #[cfg(feature = "unsize")]
    pub fn push_back<U: marker::Unsize<T>>(&mut self, v: U) -> Result<(), U>
    where
        (U,Self): crate::AlignmentValid,
    {
        self.push_back_stable(v, |p| p)
    }

    /// Push a value to the end of the list (without using `Unsize`)
    pub fn push_back_stable<U, F: FnOnce(&U) -> &T>(&mut self, v: U, f: F) -> Result<(), U>
    where
        (U,Self): crate::AlignmentValid,
    {
        <(U,Self) as crate::AlignmentValid>::check();

        // SAFE: Destination address is valid
        unsafe {
            match self.push_inner(crate::check_fat_pointer(&v, f))
            {
            Ok( (_,data) ) => {
                ptr::write(data.as_mut_ptr() as *mut U, v);
                Ok(())
            }
            Err(_) => Err(v),
            }
        }
    }

    /// Push an item to the list (setting metadata based on `fat_ptr`)
    /// UNSAFE: Caller must fill the buffer before any potential panic
    unsafe fn push_inner(&mut self, fat_ptr: &T) -> Result<(&mut [D::Inner],&mut [D::Inner]), ()> {
        let bytes = mem::size_of_val(fat_ptr);
        let words = D::round_to_words(bytes) + Self::meta_words();

        // 1. Check if there's space for the item
        if self.space_words() < words {
            // 2. If not, check if compaction would help
            if self.space_words() + self.read_pos >= words {
                self.compact();
            }
            // 3. Then, try expanding
            if self.space_words() < words {
                if let Err(_) = self.data.extend(self.write_pos + words) {
                    // if expansion fails, return error
                    return Err( () );
                }
            }
        }
        assert!( self.space_words() >= words );

        // Get the base pointer for the new item
        let slot = &mut self.data.as_mut()[self.write_pos..][..words];
        self.write_pos += words;
        let (meta, rv) = slot.split_at_mut(Self::meta_words());

        // Populate the metadata
        let mut ptr_raw: *const T = fat_ptr;
        super::store_metadata(meta, &crate::ptr_as_slice(&mut ptr_raw)[1..]);

        // Increment offset and return
        Ok( (meta, rv) )
    }

    /// Compact the list (moving the read position to zero)
    pub fn compact(&mut self) {
        if self.read_pos != 0 {
            self.data.as_mut().rotate_left(self.read_pos);
            self.write_pos -= self.read_pos;
            self.read_pos = 0;
        }
    }


    /// Checks if the queue is currently empty
    pub fn empty(&self) -> bool {
        self.read_pos == self.write_pos
    }

    /// Remove an item from the front of the list
    pub fn pop_front(&mut self) -> Option<PopHandle<T,D>> {
        if self.read_pos == self.write_pos {
            None
        }
        else {
            Some(PopHandle { parent: self })
        }
    }
    /// Peek the front of the queue
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.read_pos == self.write_pos {
            None
        }
        else {
            Some(unsafe { &mut *self.front_raw() })
        }
    }
    /// Peek the front of the queue
    pub fn front(&self) -> Option<&T> {
        if self.read_pos == self.write_pos {
            None
        }
        else {
            Some(unsafe { &*self.front_raw() })
        }
    }

    /// Obtain an immutable iterator (yields references to items, in insertion order)
    /// ```
    /// let mut list = ::stack_dst::FifoA::<str, [usize; 8]>::new();
    /// list.push_back_str("Hello");
    /// list.push_back_str("world");
    /// let mut it = list.iter();
    /// assert_eq!(it.next(), Some("Hello"));
    /// assert_eq!(it.next(), Some("world"));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter(&self) -> Iter<T,D> {
        Iter(self, self.read_pos)
    }
    /// Obtain a mutable iterator
    /// ```
    /// let mut list = ::stack_dst::FifoA::<[u8], [usize; 8]>::new();
    /// list.push_copied(&[1,2,3]);
    /// list.push_copied(&[9]);
    /// for v in list.iter_mut() {
    ///     v[0] -= 1;
    /// }
    /// let mut it = list.iter();
    /// assert_eq!(it.next(), Some(&[0,2,3][..]));
    /// assert_eq!(it.next(), Some(&[8][..]));
    /// assert_eq!(it.next(), None);
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<T,D> {
        IterMut(self, self.read_pos)
    }
    // Note: No into_iter, not possible due to unsized types
    // Could make a `drain` that returns read handles (pops as it goes)
    
    fn front_raw(&self) -> *mut T {
        assert!(self.read_pos < self.write_pos);

        // SAFE: Internal consistency maintains the metadata validity
        unsafe { self.raw_at(self.read_pos) }
    }
    // UNSAFE: Caller must ensure that `pos` is the start of an object
    unsafe fn raw_at(&self, pos: usize) -> *mut T {
        assert!(pos >= self.read_pos);
        assert!(pos < self.write_pos);
        let meta = &self.data.as_ref()[pos..];
        let mw = Self::meta_words();
        super::make_fat_ptr(meta[mw..].as_ptr() as usize, &meta[..mw])
    }
    fn pop_front_inner(&mut self) {
        // SAFE: `front_raw` asserts that there's an item, rest is correct
        unsafe {
            let ptr = &mut *self.front_raw();
            let len = mem::size_of_val(ptr);
            ptr::drop_in_place(ptr);
            let words = D::round_to_words(len);
            self.read_pos += Self::meta_words() + words;
        }
    }
}
impl<D: ::DataBuf> FifoA<str, D> {
    /// Push the contents of a string slice as an item onto the stack
    pub fn push_back_str(&mut self, v: &str) -> Result<(), ()> {
        unsafe {
            self.push_inner(v)
                .map(|(_,d)| ptr::copy(v.as_bytes().as_ptr(), d.as_mut_ptr() as *mut u8, v.len()))
        }
    }
}
impl<D: ::DataBuf, T: Clone> FifoA<[T], D> {
    /// Pushes a set of items (cloning out of the input slice)
    pub fn push_cloned(&mut self, v: &[T]) -> Result<(), ()> {
        // SAFE: Carefully constructed to maintain consistency
        unsafe {
            let (meta,d) = self.push_inner(v)?;
            crate::list_push_cloned(meta, d, v);
        }

        Ok( () )
    }
    /// Pushes a set of items (copying out of the input slice)
    pub fn push_copied(&mut self, v: &[T]) -> Result<(), ()>
    where
        T: Copy
    {
        // SAFE: Carefully constructed to maintain consistency
        unsafe {
            self.push_inner(v)
                .map(|(_,d)| ptr::copy(v.as_ptr() as *const u8, d.as_mut_ptr() as *mut u8, mem::size_of_val(v)))
        }
    }
}

impl<T: ?Sized, D: crate::DataBuf> ops::Drop for FifoA<T, D> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {
        }
    }
}

/// Handle returned by `FifoA::pop` (does the actual pop on drop)
pub struct PopHandle<'a, T: 'a + ?Sized, D: 'a + crate::DataBuf> {
    parent: &'a mut FifoA<T, D>,
}
impl<'a, T: ?Sized, D: crate::DataBuf> ops::Deref for PopHandle<'a, T, D> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.parent.front_raw() }
    }
}
impl<'a, T: ?Sized, D: crate::DataBuf> ops::DerefMut for PopHandle<'a, T, D> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.parent.front_raw() }
    }
}
impl<'a, T: ?Sized, D: crate::DataBuf> ops::Drop for PopHandle<'a, T, D> {
    fn drop(&mut self) {
        self.parent.pop_front_inner();
    }
}

/// DST FIFO iterator (immutable)
pub struct Iter<'a, T: 'a + ?Sized, D: 'a + crate::DataBuf>( &'a FifoA<T, D>, usize );
impl<'a, T: 'a + ?Sized, D: 'a + crate::DataBuf> iter::Iterator for Iter<'a, T, D> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.1 == self.0.write_pos {
            None
        }
        else {
            // SAFE: Bounds checked, aliasing enforced by API
            let rv = unsafe { &*self.0.raw_at(self.1) };
            self.1 += FifoA::<T,D>::meta_words() + D::round_to_words(mem::size_of_val(rv));
            Some(rv)
        }
    }
}
/// DST FIFO iterator (mutable)
pub struct IterMut<'a, T: 'a + ?Sized, D: 'a + crate::DataBuf>( &'a mut FifoA<T, D>, usize );
impl<'a, T: 'a + ?Sized, D: 'a + crate::DataBuf> iter::Iterator for IterMut<'a, T, D> {
    type Item = &'a mut T;
    fn next(&mut self) -> Option<&'a mut T> {
        if self.1 == self.0.write_pos {
            None
        }
        else {
            // SAFE: Bounds checked, aliasing enforced by API
            let rv = unsafe { &mut *self.0.raw_at(self.1) };
            self.1 += FifoA::<T,D>::meta_words() + D::round_to_words(mem::size_of_val(rv));
            Some(rv)
        }
    }
}

