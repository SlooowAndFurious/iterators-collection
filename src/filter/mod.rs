//! The iterators in this module aim to select the elements to yield

/// Excludes an object from iteration. Based on a blacklist
/// 
/// # Example
/// ```
/// use iterators_collection::filter::Exclude;
/// 
/// let array = [1, 2, 3, 4, 5];
/// let iter = array.iter().cloned();
/// // The iterator will ignore the values 3 and 5
/// let mut iter = Exclude::with_blacklist(iter, vec![3, 5]);
/// 
/// // Once 3 and 5 removed, there are only 1, 2 and 4
/// assert_eq!(iter.collect::<Vec<i32>>(), vec![1, 2, 4]);
/// ```
#[derive(Clone)]
pub struct Exclude<T>
where
    T: Iterator,
    T::Item: PartialEq,
{
    excluded: Vec<T::Item>,
    cur: T,
}

impl<T> Exclude<T>
where
    T: Iterator,
    T::Item: PartialEq,
{
    /// Returns a new object with an empty blacklist
    pub fn new(iterator: T) -> Self {
        Exclude {
            cur: iterator,
            excluded: Vec::new(),
        }
    }

    /// Returns a new object with the given blacklist
    pub fn with_blacklist(iterator: T, blacklist: Vec<T::Item>) -> Self {
        Exclude {
            cur: iterator,
            excluded: blacklist,
        }
    }

    /// Adds the object passed as arguments to the blacklist. It will be added only if it is not already inside the blacklist
    pub fn exclude(&mut self, new: T::Item) {
        if self.excluded.iter().position(|x| x == &new).is_none() {
            self.force_exclude(new);
        }
    }

    /// Forces the object passed as arguments to be pushed to the blacklist. It will be added even if already present. You may want to avoid this behaviour and use `exclude` instead. Use it only if you are sure this element is not in the blacklist or if `T::Item` is designed as `a == b`, `b == c` but `a != c`. However, it is safe to use it. It may just lead to performance issues
    pub fn force_exclude(&mut self, new: T::Item) {
        self.excluded.push(new);
    }

    /// Returns the iterator in use
    pub fn get_iterator(&self) -> &T {
        &self.cur
    }

    /// Returns the iterator in use as a mutable reference
    pub fn get_mut_iterator(&mut self) -> &mut T {
        &mut self.cur
    }
}

impl<T> Iterator for Exclude<T>
where
    T: Iterator,
    T::Item: PartialEq,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.cur.next() {
                // Happens when the iterator is fully consumed
                None    => return None,

                Some(i) => if self.excluded.iter().position(|x| x == &i).is_none() {
                               return Some(i);
                },
            }
        }
    }
}

impl<T> crate::ResettableIterator for Exclude<T>
where
    T: crate::ResettableIterator,
    T::Item: PartialEq,
{
    fn reset(&mut self) {
        self.cur.reset();
    }
}

impl<T> crate::child::ChildIterator for Exclude<T>
where
    T: Iterator,
    T::Item: PartialEq,
{
    type Parent = T;

    fn release_parent(self) -> Self::Parent {
        self.cur
    }

    fn get_parent_mut(&mut self) -> &mut Self::Parent {
        &mut self.cur
    }

    fn get_parent(&self) -> &Self::Parent {
        &self.cur
    }
}

#[cfg(test)]
mod tests;
