//! A module about advanced memory sharing during iteration

/// Iterates twice over the same collection
/// 
/// # Example
/// The following code
/// 
/// ```
/// use iterators_collection::share::DoubleIterator;
/// 
/// let mut array = [1, 2, 3, 4, 5];
/// let iter = DoubleIterator::new(&mut array);
/// 
/// for (i, j) in iter {
///     // Some code here
/// }
/// ```
/// 
/// Means the same as
/// 
/// ```
/// let array = [1, 2, 3, 4, 5];
/// for i in array.iter() {
///     for j in array.iter() {
///         // Some code here
///     }
/// }
/// ```
/// 
/// with some differences:
/// - i and j will never be the same with `DoubleIterator`
/// 
/// - you can safely iterate on a mutable slice with `DoubleIterator`
/// 
/// - i and j CANNOT be shared across threads because it is unsafe to increment the iterator in one thread while accessing one of these references from the other one. It may lead to a data race
/// 
/// - i and j are raw pointers and not references because the correct lifetime for the borrowed values is not known at compile time since a simple call to the `next` method may lead to a data race because two mutable references to the same object may exist
/// 
/// Since version 0.3.0, the preferred way to do it is to use the `safe_for_each` method because you can use this iterator without writting unsafe code
/// ```
/// use iterators_collection::share::DoubleIterator;
/// 
/// let mut array = [1, 2, 3, 4, 5];
/// let iter = DoubleIterator::new(&mut array);
/// 
/// iter.safe_for_each(|i, j| {
///     // Some code here
/// });
/// ```
pub struct DoubleIterator<'a, T> {
    slice: &'a mut [T],
    first: usize,
    second: usize,
}

impl<'a, T> DoubleIterator<'a, T> {
    /// Creates a `DoubleIterator` from a slice
    /// 
    /// # Panics
    /// Panics if `slice.len() < 2`
    pub fn new(slice: &'a mut [T]) -> Self {
        assert!(slice.len() >= 2);

        Self {
            slice,

            first: 0,
            second: 1,
        }
    }

    /// Returns a mutable pointer to the `index`th element of the borrowed slice
    /// 
    /// # Unsafety
    /// Indexes are not checked if the `debug_assert!`s are disabled
    /// 
    /// This pointer is unsafe to use
    unsafe fn nth_ptr(&mut self, index: usize) -> *mut T {
        debug_assert!(index < self.slice.len());
        self.slice.get_unchecked_mut(index) as *mut T
    }

    /// Increments the indexes `first` and `second` or returns Err
    fn increment(&mut self) -> Result<(), ()> {
        loop {
            // Increment
            self.second += 1;

            // Check for overflow
            if self.second == self.slice.len() {
                self.second = 0;
                self.first += 1;

                if self.first >= self.slice.len() {
                    return Err(());
                }
            }

            if self.first != self.second {
                return Ok(());
            }
        }
    }

    /// Runs the given closure in a safe context
    /// 
    /// # Example
    /// ```
    /// use iterators_collection::share::DoubleIterator;
    /// 
    /// let mut array = [1, 2, 3, 4, 5];
    /// let iter = DoubleIterator::new(&mut array);
    /// 
    /// iter.safe_for_each(|i, j| {
    ///     println!("Got i = {} and j = {}", i, j);
    ///     assert_ne!(i, j);
    /// });
    /// ```
    /// 
    /// # Notes
    /// Not like a legacy iteration using a `for` loop, i and j are references because it's safe to use in this context
    pub fn safe_for_each<F: Fn(&mut T, &mut T)>(self, callback: F) {
        for (i, j) in self {
            unsafe {
                callback(&mut *i, &mut *j);
            }
        }
    }

    /// Sets the position of the iterator
    /// 
    /// # Parameters
    /// `i` the position of the first pointer of the tuple returned by the `Iterator` trait's implementation
    /// 
    /// `j` the position of the second one
    /// 
    /// # Panics
    /// Panics if either `i` or `j` are out of range (greater or equal to `slice.len()`)
    /// 
    /// Panics if `i == j`
    pub fn set(&mut self, i: usize, j: usize) {
        assert_ne!(i, j);
        assert!(i < self.slice.len() && j < self.slice.len());

        self.first = i;
        self.second = j;
    }
}

impl<T> crate::ResettableIterator for DoubleIterator<'_, T> {
    fn reset(&mut self) {
        self.first = 0;
        self.second = 1;
    }
}

impl<T> Iterator for DoubleIterator<'_, T> {
    type Item = (*mut T, *mut T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.first == self.slice.len() {
            return None;
        }

        let returned = Some(unsafe { (self.nth_ptr(self.first), self.nth_ptr(self.second)) });
        std::mem::drop(self.increment()); // Dropping is a way to ignore the error which doesn't matter here

        returned
    }
}

/// A `DoubleIterator` iterating on one single "line" (see explanation below)
/// 
/// # Introduction
/// The iteration cycle of a `DoubleIterator` can be seen as a matrix with `i` as the value of the line and `j` as the value of the collumn. For example, in an array as `[0, 1, 2, 3, 4]`, a `DoubleIterator` will return the numbered cells and discard the blanks
/// ```text
/// +---+---+---+---+---+---+
/// |   |i=0|i=1|i=2|i=3|i=4|
/// +---+---+---+---+---+---+
/// |j=0|   | 1 | 2 | 3 | 4 |
/// +---+---+---+---+---+---+
/// |j=1| 5 |   | 6 | 7 | 8 |
/// +---+---+---+---+---+---+
/// |j=2| 9 |10 |   |11 |12 |
/// +---+---+---+---+---+---+
/// |j=3|13 |14 |15 |   |16 |
/// +---+---+---+---+---+---+
/// |j=4|17 |18 |19 |20 |   |
/// +---+---+---+---+---+---+
/// ```
/// 
/// In this example, the first iterator tuple returned (once the two members dereferenced) is `(0, 1)`, then `(0, 2)`, `(0, 3)`, `(0, 4)`, and at the end of line, `i` is reset and `j` is incrememented, returning `(1, 0)`, `(1, 2)` because there is a blank in the cell (`i=1`; `j=1`), `(1, 3)`...
/// 
/// The blanks are here because there can't be two mutable references on the same object.
/// 
/// But in some cases, you need to iterate only on one single line and not the whole grid, that's why `SingleLineIterator` exists.
/// 
/// # Hey, wait! Why using that iterator and not simply iterating manually?
/// Just like `DoubleIterator` does, this iterator returns two raw pointers to a member of the slice to iterate
/// 
/// Since version 0.3.3, the prefered way to do this is to use the `safe_for_each` method
pub struct SingleLineIterator<'a, T> {
    slice: &'a mut [T],
    index: usize,
    cur: usize,
}

impl<'a, T> SingleLineIterator<'a, T> {
    /// Returns a new `SingleLineIterator` which returns a tuple of a mutable reference to `slice[index]` and to another member of `slice` at each iteration
    /// 
    /// # Panics
    /// Panics if `index` is greater or equal to `slice.len()`
    pub fn new(slice: &'a mut [T], index: usize) -> Self {
        assert!(index < slice.len());

        Self {
            slice,
            index,
            cur: if index == 0 {
                1
            } else {
                0
            },
        }
    }

    /// Runs the given closure in a safe context
    /// 
    /// # Example
    /// ```
    /// use iterators_collection::share::SingleLineIterator;
    /// 
    /// let mut array = [1, 2, 3, 4, 5];
    /// let iter = SingleLineIterator::new(&mut array, 0);
    /// 
    /// iter.safe_for_each(|i, j| {
    ///     println!("Got i = {} and j = {}", i, j);
    ///     assert_ne!(i, j);
    /// });
    /// ```
    /// 
    /// # Notes
    /// Not like a legacy iteration using a `for` loop, i and j are references because it's safe to use in this context
    pub fn safe_for_each<F: Fn(&mut T, &mut T)>(self, callback: F) {
        for (i, j) in self {
            unsafe {
                callback(&mut *i, &mut *j);
            }
        }
    }
}

impl<T> crate::ResettableIterator for SingleLineIterator<'_, T> {
    fn reset(&mut self) {
        self.cur = 0;
    }
}

impl<'a, T> Iterator for SingleLineIterator<'a, T> {
    type Item = (*mut T, *mut T);

    fn next(&mut self) -> Option<Self::Item> {
        let returned = if self.cur >= self.slice.len() {
            None
        } else {
            unsafe {
                let ptr1 = self.slice.get_unchecked_mut(self.index) as *mut T;
                let ptr2 = self.slice.get_unchecked_mut(self.cur)   as *mut T;

                Some((ptr1, ptr2))
            }
        };

        self.cur += 1;
        if self.cur == self.index {
            self.cur += 1;
        }

        returned
    }
}

impl<'a, T> From<DoubleIterator<'a, T>> for SingleLineIterator<'a, T> {
    fn from(src: DoubleIterator<'a, T>) -> Self {
        Self {
            cur: src.second,
            index: src.first,
            slice: src.slice,
        }
    }
}


#[cfg(test)]
mod tests;
