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
        if n >= self.buffer.len() {
            for _ in 0..=(n - self.buffer.len()) {
                self.buffer.push_front(self.inner.next()?);
            }
        }
        Some(&self.buffer[self.buffer.len() - n - 1])
    }

    /// Returns a lookahead iterator that will peek successive elements without
    /// consuming the original one.
    ///
    /// For the reason why this requires [`Clone`], see [`LookaheadIterator`].
    pub fn lookahead(&mut self) -> LookaheadIterator<'_, I>
    where
        I::Item: Clone,
    {
        LookaheadIterator {
            inner: self,
            pos: 0,
        }
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

/// A lookahead iterator that doesn't consume the borrowed [`PushBackIterator`].
///
/// Unfortunately, we have to require [`Clone`] for the item type, because the
/// [`Iterator`] trait, as it is defined in the standard library, does not
/// allow us to return a reference that mutably borrows the iterator itself.
///
/// This borrow is needed because it is not safe to advance the iterator when
/// the previous returned item is still alive, as this item is borrowing the
/// VecDeque that we have to mutate.
///
pub struct LookaheadIterator<'i, I: Iterator> {
    inner: &'i mut PushBackIterator<I>,
    pos: usize,
}

impl<'i, I: Iterator> Iterator for LookaheadIterator<'i, I>
where
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let next = self.inner.peek_nth(self.pos);
        self.pos += 1;
        next.cloned()
    }
}

// Implemented by hand because Derive macro fails for some reason.
impl<'i, I> std::fmt::Debug for LookaheadIterator<'i, I>
where
    I: Iterator + std::fmt::Debug,
    I::Item: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LookaheadIterator")
            .field("inner", &self.inner)
            .field("pos", &self.pos)
            .finish()
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

    #[test]
    fn lookahead_iterator() {
        let items = vec![0, 1, 2, 3, 4, 5];
        let mut iter = PushBackIterator::from(items.into_iter());

        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));

        let lookahead = iter.lookahead();

        assert_eq!(lookahead.take(2).collect::<Vec<_>>(), vec![3, 4]);
        assert_eq!(iter.collect::<Vec<_>>(), vec![3, 4, 5]);
    }
}
