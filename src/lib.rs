//! This create providers an implementation of `PushBackIterator` which is a
//! wrapper around an iterator which allows for items to be pushed back onto
//! the iterator to be consumed on subsequent call to `next()`.
//!
//! ```
//! use pushback_iter::PushBackIterator;
//!
//! let items = vec![1, 2, 3];
//! let mut iter = PushBackIterator::from(items.into_iter());
//!
//! let item = iter.next().unwrap();
//! assert_eq!(item, 1);
//!
//! iter.push_back(item);
//! assert_eq!(iter.next(), Some(1));
//!
//! iter.push_back(6);
//! iter.push_back(5);
//! assert_eq!(iter.next(), Some(5));
//! assert_eq!(iter.next(), Some(6));
//! assert_eq!(iter.next(), Some(2));
//! ```

#![deny(
    missing_docs,
    missing_debug_implementations,
    unreachable_pub,
    broken_intra_doc_links
)]
#![warn(rust_2018_idioms)]

use std::collections::VecDeque;

/// An iterator with a `push_back(item)` method that allows
/// items to be pushed back onto the iterator.
///
/// [`Iterator`]: trait.Iterator.html
#[derive(Debug, Clone)]
pub struct PushBackIterator<I: Iterator> {
    buffer: VecDeque<I::Item>,
    inner: I,
}

impl<I: Iterator> PushBackIterator<I> {
    /// Push back an item to the beginning of the iterator.
    ///
    /// Items pushed back onto the iterator are returned from [`next`] in a
    /// last-in-first out basis.
    pub fn push_back(&mut self, item: I::Item) {
        self.buffer.push_back(item)
    }

    /// Returns a reference to the next() value without advancing the iterator.
    ///
    /// Like [`next`], if there is a value, it is wrapped in a `Some(T)`.
    /// But if the iteration is over, `None` is returned.
    #[inline]
    pub fn peek(&mut self) -> Option<&I::Item> {
        self.peek_nth(0)
    }

    /// Returns a reference to the nth(n) value without advancing the iterator.
    ///
    /// Like [`nth`], if there is a value, it is wrapped in a `Some(T)`.
    /// But if the iteration is over, `None` is returned.
    pub fn peek_nth(&mut self, n: usize) -> Option<&I::Item> {
        self.peek_nth_mut(n).map(|p| &*p)
    }

    /// Like [`peek_nth`] but returns a mutable reference.
    pub fn peek_nth_mut(&mut self, n: usize) -> Option<&mut I::Item> {

        // PANIC SAFETY: this loop is entered if len <= n. At the end of the loop, len > n.
        for _ in self.buffer.len() ..= n {
            self.buffer.push_front(self.inner.next()?);
        }

        let len = self.buffer.len();

        // PANIC SAFETY: this is safe because len > n
        Some(&mut self.buffer[len - n - 1])
    }

    /// Reserves capacity for at least `additional` more elements in the push back buffer
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `usize`.
    ///
    pub fn reserve(&mut self, additional: usize) {
        self.buffer.reserve(additional)
    }

    /// Shrinks the capacity of the buffer as much as possible.
    ///
    pub fn shrink_to_fit(&mut self) {
        self.buffer.shrink_to_fit()
    }
}

impl<I: Iterator> From<I> for PushBackIterator<I> {
    fn from(inner: I) -> Self {
        PushBackIterator {
            buffer: VecDeque::new(),
            inner,
        }
    }
}

impl<I: Iterator> Iterator for PushBackIterator<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.buffer.pop_back().or_else(|| self.inner.next())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lower, upper) = self.inner.size_hint();
        (
            lower + self.buffer.len(),
            upper.map(|upper| upper + self.buffer.len()),
        )
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.buffer.len() + self.inner.count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.inner.last()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n < self.buffer.len() {
            self.buffer.truncate(self.buffer.len() - n);
            self.buffer.pop_back()
        } else {
            let n = n - self.buffer.len();
            self.buffer.clear();
            self.inner.nth(n)
        }
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for PushBackIterator<I> {
    fn len(&self) -> usize {
        self.buffer.len() + self.inner.len()
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for PushBackIterator<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().or_else(|| self.buffer.pop_front())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_back() {
        let items = vec![0, 1, 2, 3];
        let mut iter = PushBackIterator::from(items.into_iter());
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        iter.push_back(1);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        iter.push_back(2);
        iter.push_back(1);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));

        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);

        iter.push_back(0);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn peek_nth() {
        let items = vec![0, 1, 2, 3, 4, 5];
        let mut iter = PushBackIterator::from(items.into_iter());
        assert_eq!(iter.peek_nth(2), Some(&2));
        assert_eq!(iter.peek_nth(2), Some(&2));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.peek_nth(2), Some(&3));
        iter.push_back(0);
        assert_eq!(iter.peek_nth(2), Some(&2));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
    }
}
