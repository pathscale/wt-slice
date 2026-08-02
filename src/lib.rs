// Copyright 2017 Alkis Evlogimenos
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![no_std]

//! This crate provides extensions for [`slice`]s.
//! 
//! # Examples
//! 
//! ```
//! use wt_slice::*;
//! 
//! let b = [1, 3];
//! 
//! assert_eq!(b.lower_bound(&1), 0);
//! 
//! assert_eq!(b.upper_bound(&1), 1);
//! 
//! assert_eq!(b.equal_range(&3), 1..2);
//! ```
//! 
//! [`slice`]: https://doc.rust-lang.org/stable/std/primitive.slice.html
use core::cmp::Ordering::{self, Less, Greater};

/// Exact search for ordered slices with an early equality exit and branchless
/// search-direction selection.
///
/// This combines the useful properties of a conventional early-exit binary
/// search with the conditional selection used by modern standard-library
/// binary search implementations. It returns either a matching index or the
/// position where the value can be inserted while preserving order.
pub trait ExactSearch {
    type Item;

    /// Searches for `x`, returning a matching index or its insertion position.
    fn exact_search(&self, x: &Self::Item) -> Result<usize, usize>
    where
        Self::Item: Ord;

    /// Searches with a comparator, returning a match or insertion position.
    fn exact_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a Self::Item) -> Ordering;

    /// Searches by an extracted key, returning a match or insertion position.
    fn exact_search_by_key<'a, K, F>(&'a self, key: &K, f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a Self::Item) -> K,
        K: Ord;
}

impl<T> ExactSearch for [T] {
    type Item = T;

    #[inline]
    fn exact_search(&self, x: &Self::Item) -> Result<usize, usize>
    where
        Self::Item: Ord,
    {
        self.exact_search_by(|candidate| candidate.cmp(x))
    }

    #[inline]
    fn exact_search_by<'a, F>(&'a self, mut f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a Self::Item) -> Ordering,
    {
        let mut size = self.len();
        if size == 0 {
            return Err(0);
        }

        let mut base = 0;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;

            // SAFETY: `base + size` never exceeds `self.len()`, and
            // `half < size`, so `mid` is always a valid index.
            let cmp = f(unsafe { self.get_unchecked(mid) });
            if cmp == Ordering::Equal {
                return Ok(mid);
            }

            // Search direction is deliberately expressed as an unpredictable
            // selection so supported targets use a conditional move rather
            // than a hard-to-predict left/right branch.
            base = core::hint::select_unpredictable(cmp == Greater, base, mid);
            size -= half;
        }

        // SAFETY: a non-empty search retains `base < self.len()`.
        match f(unsafe { self.get_unchecked(base) }) {
            Ordering::Less => Err(base + 1),
            Ordering::Equal => Ok(base),
            Ordering::Greater => Err(base),
        }
    }

    #[inline]
    fn exact_search_by_key<'a, K, F>(&'a self, key: &K, mut f: F) -> Result<usize, usize>
    where
        F: FnMut(&'a Self::Item) -> K,
        K: Ord,
    {
        self.exact_search_by(|candidate| f(candidate).cmp(key))
    }
}

/// Extends [`slice`] with fast operations on ordered slices.
/// 
/// [`slice`]: https://doc.rust-lang.org/stable/std/primitive.slice.html
pub trait Ext {
    type Item;

    /// Returns the index `i` pointing to the first element in the ordered slice
    /// that is _not less_ than `x`.
    /// 
    /// The slice MUST be ordered by the order defined by its elements.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let a = [10, 11, 13, 13, 15];
    /// assert_eq!(a.lower_bound(&9), 0);
    /// assert_eq!(a.lower_bound(&10), 0);
    /// assert_eq!(a.lower_bound(&11), 1);
    /// assert_eq!(a.lower_bound(&12), 2);
    /// assert_eq!(a.lower_bound(&13), 2);
    /// assert_eq!(a.lower_bound(&14), 4);
    /// assert_eq!(a.lower_bound(&15), 4);
    /// assert_eq!(a.lower_bound(&16), 5);
    /// ```
    fn lower_bound(&self, x: &Self::Item) -> usize
    where
        Self::Item: Ord;

    /// Returns the index `i` pointing to the first element in the ordered slice
    /// for which `f(self[i]) != Less`.
    /// 
    /// The slice MUST be ordered by the order defined by the comparator
    /// function. The comparator function should take an element and return
    /// `Ordering` that is consistent with the ordering of the slice.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let b = [1, 2, 3, 6, 9, 9];
    /// assert_eq!(b.lower_bound(&3), b.lower_bound_by(|x| x.cmp(&3)));
    /// ```
    fn lower_bound_by<'a, F>(&'a self, f: F) -> usize
    where
        F: FnMut(&'a Self::Item) -> Ordering;

    /// Returns the index `i` pointing to the first element in the ordered slice
    /// for which `f(self[i]) >= k`.
    /// 
    /// The slice MUST be ordered by the order defined by the keys of its
    /// elements.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let b = [1, 2, 3, 6, 9, 9];
    /// assert_eq!(b.lower_bound(&3), b.lower_bound_by_key(&6, |x| x * 2));
    /// ```
    fn lower_bound_by_key<'a, K, F>(&'a self, k: &K, f: F) -> usize
    where
        F: FnMut(&'a Self::Item) -> K,
        K: Ord;

    /// Returns the index `i` pointing to the first element in the ordered slice
    /// that is _greater_ than `x`.
    /// 
    /// The slice MUST be ordered by the order defined by its elements.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let a = [10, 11, 13, 13, 15];
    /// assert_eq!(a.upper_bound(&9), 0);
    /// assert_eq!(a.upper_bound(&10), 1);
    /// assert_eq!(a.upper_bound(&11), 2);
    /// assert_eq!(a.upper_bound(&12), 2);
    /// assert_eq!(a.upper_bound(&13), 4);
    /// assert_eq!(a.upper_bound(&14), 4);
    /// assert_eq!(a.upper_bound(&15), 5);
    /// assert_eq!(a.upper_bound(&16), 5);
    /// ```
    fn upper_bound(&self, x: &Self::Item) -> usize
    where
        Self::Item: Ord;

    /// Returns the index `i` pointing to the first element in the ordered slice
    /// for which `f(self[i]) == Greater`.
    /// 
    /// The slice MUST be ordered by the order defined by the comparator
    /// function. The comparator function should take an element and return
    /// `Ordering` that is consistent with the ordering of the slice.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let b = [1, 2, 3, 6, 9, 9];
    /// assert_eq!(b.upper_bound(&3), b.upper_bound_by(|x| x.cmp(&3)));
    /// ```
    fn upper_bound_by<'a, F>(&'a self, f: F) -> usize
    where
        F: FnMut(&'a Self::Item) -> Ordering;

    /// Returns the index `i` pointing to the first element in the ordered slice
    /// for which `f(self[i]) > k`.
    /// 
    /// The slice MUST be ordered by the order defined by the keys of its
    /// elements.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let b = [1, 2, 3, 6, 9, 9];
    /// assert_eq!(b.lower_bound(&3), b.lower_bound_by_key(&6, |x| x * 2));
    fn upper_bound_by_key<'a, K, F>(&'a self, k: &K, f: F) -> usize
    where
        F: FnMut(&'a Self::Item) -> K,
        K: Ord;

    /// Returns the [`Range`] `a..b` such that all elements in `self[a..b]` are
    /// _equal_ to `x`.
    /// 
    /// The slice MUST be ordered by the order defined by its elements.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let b = [10, 11, 13, 13, 15];
    /// for i in 9..17 {
    ///     assert_eq!(b.equal_range(&i), (b.lower_bound(&i)..b.upper_bound(&i)));
    /// }
    /// ```
    /// [`Range`]: https://doc.rust-lang.org/stable/std/ops/struct.Range.html
    fn equal_range(&self, x: &Self::Item) -> core::ops::Range<usize>
    where
        Self::Item: Ord;
    
    /// Returns the [`Range`] `a..b` such that for all elements `e` in `self[a..b]` 
    /// `f(e) == Equal`.
    ///
    /// The slice MUST be ordered by the order defined by the comparator
    /// function. The comparator function should take an element and return
    /// `Ordering` that is consistent with the ordering of the slice.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let b = [10, 11, 13, 13, 15];
    /// for i in 9..17 {
    ///     assert_eq!(b.equal_range(&i), b.equal_range_by(|x| x.cmp(&i)));
    /// }
    /// ```
    /// [`Range`]: https://doc.rust-lang.org/stable/std/ops/struct.Range.html
    fn equal_range_by<'a, F>(&'a self, f: F) -> core::ops::Range<usize>
    where
        F: FnMut(&'a Self::Item) -> Ordering;
    
    /// Returns the [`Range`] `a..b` such that for all elements `e` in `self[a..b]` 
    /// `f(e) == k`.
    ///
    /// The slice MUST be ordered by the order defined by the keys of its
    /// elements.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let b = [10, 11, 13, 13, 15];
    /// for i in 9..17 {
    ///     let i2 = i * 2;
    ///     assert_eq!(b.equal_range(&i), b.equal_range_by_key(&i2, |x| x * 2));
    /// }
    /// ```
    /// [`Range`]: https://doc.rust-lang.org/stable/std/ops/struct.Range.html
    fn equal_range_by_key<'a, K, F>(&'a self, k: &K, f: F) -> core::ops::Range<usize>
    where
        F: FnMut(&'a Self::Item) -> K,
        K: Ord;

    /// Transforms the slice into the next permutation from the set of all
    /// permutations that are lexicographically ordered with respect to the
    /// natural order of T. Returns true if such permutation exists, otherwise
    /// transforms the range into the first permutation and returns false.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let mut b = [2, 1, 3];
    /// let mut v = Vec::new();
    /// for _ in 0..6 {
    ///     let x = b.next_permutation();
    ///     v.push((x, b.to_vec()));
    /// }
    /// assert_eq!(v, &[(true, [2, 3, 1].to_vec()),
    ///                 (true, [3, 1, 2].to_vec()),
    ///                 (true, [3, 2, 1].to_vec()),
    ///                 (false, [1, 2, 3].to_vec()),
    ///                 (true, [1, 3, 2].to_vec()),
    ///                 (true, [2, 1, 3].to_vec())]);
    fn next_permutation(&mut self) -> bool
    where
        Self::Item: Ord;

    /// Transforms the slice into the previous permutation from the set of all
    /// permutations that are lexicographically ordered with respect to the
    /// natural order of T. Returns true if such permutation exists, otherwise
    /// transforms the range into the last permutation and returns false.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let mut b = [2, 1, 3];
    /// let mut v = Vec::new();
    /// for _ in 0..6 {
    ///     let x = b.prev_permutation();
    ///     v.push((x, b.to_vec()));
    /// }
    /// assert_eq!(v, &[(true, [1, 3, 2].to_vec()),
    ///                 (true, [1, 2, 3].to_vec()),
    ///                 (false, [3, 2, 1].to_vec()),
    ///                 (true, [3, 1, 2].to_vec()),
    ///                 (true, [2, 3, 1].to_vec()),
    ///                 (true, [2, 1, 3].to_vec())]);
    fn prev_permutation(&mut self) -> bool
    where
        Self::Item: Ord;

    /// Applies `permutation` to the slice. For each element at index `i` the
    /// following holds:
    /// 
    ///   new_self[i] == old_self[permutation[i]]
    ///
    /// The transformation happens in O(N) time and O(1) space. `permutation`
    /// is mutated during the transformation but it is restored to its original
    /// state on return.
    /// 
    /// # Panics
    /// 
    /// This function panics if `self` and `permutation` do not have the same
    /// length or any value in `permutation` is not in `0..self.len()`.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let mut b = ['d', 'a', 'c', 'b'];
    /// let mut p = [1, 3, 2, 0];
    /// b.apply_permutation(&mut p);
    /// assert_eq!(b, ['a', 'b', 'c', 'd']);
    /// assert_eq!(p, [1, 3, 2, 0]);
    fn apply_permutation(&mut self, permutation: &mut [isize]);

    /// Applies the inverse of `permutation` to the slice. For each element at
    /// index `i` the following holds:
    /// 
    ///   new_self[permutation[i]] == old_self[i]
    ///
    /// The transformation happens in O(N) time and O(1) space. `permutation`
    /// is mutated during the transformation but it is restored to its original
    /// state on return.
    /// 
    /// # Panics
    /// 
    /// This function panics if `self` and `permutation` do not have the same
    /// length or any value in `permutation` is not in `0..self.len()`.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let mut b = ['d', 'a', 'c', 'b'];
    /// let mut p = [3, 0, 2, 1];
    /// b.apply_inverse_permutation(&mut p);
    /// assert_eq!(b, ['a', 'b', 'c', 'd']);
    /// assert_eq!(p, [3, 0, 2, 1]);
    fn apply_inverse_permutation(&mut self, permutation: &mut [isize]);
}

impl<T> Ext for [T] {
    type Item = T;

    fn lower_bound(&self, x: &Self::Item) -> usize
    where
        T: Ord,
    {
        self.lower_bound_by(|y| y.cmp(x))
    }
    fn lower_bound_by<'a, F>(&'a self, mut f: F) -> usize
    where
        F: FnMut(&'a Self::Item) -> Ordering,
    {
        let s = self;
        let mut size = s.len();
        if size == 0 {
            return 0;
        }
        let mut base = 0usize;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            let cmp = f(unsafe { s.get_unchecked(mid) });
            base = if cmp == Less { mid } else { base };
            size -= half;
        }
        let cmp = f(unsafe { s.get_unchecked(base) });
        base + (cmp == Less) as usize
    }
    fn lower_bound_by_key<'a, K, F>(&'a self, k: &K, mut f: F) -> usize
    where
        F: FnMut(&'a Self::Item) -> K,
        K: Ord,
    {
        self.lower_bound_by(|e| f(e).cmp(k))
    }

    fn upper_bound(&self, x: &Self::Item) -> usize
    where
        T: Ord,
    {
        self.upper_bound_by(|y| y.cmp(x))
    }

    fn upper_bound_by<'a, F>(&'a self, mut f: F) -> usize
    where
        F: FnMut(&'a Self::Item) -> Ordering,
    {
        let s = self;
        let mut size = s.len();
        if size == 0 {
            return 0;
        }
        let mut base = 0usize;
        while size > 1 {
            let half = size / 2;
            let mid = base + half;
            let cmp = f(unsafe { s.get_unchecked(mid) });
            base = if cmp == Greater { base } else { mid };
            size -= half;
        }
        let cmp = f(unsafe { s.get_unchecked(base) });
        base + (cmp != Greater) as usize
    }
    fn upper_bound_by_key<'a, K, F>(&'a self, k: &K, mut f: F) -> usize
    where
        F: FnMut(&'a Self::Item) -> K,
        K: Ord,
    {
        self.upper_bound_by(|e| f(e).cmp(k))
    }

    fn equal_range(&self, x: &Self::Item) -> core::ops::Range<usize>
    where
        T: Ord,
    {
        self.equal_range_by(|y| y.cmp(x))
    }
    fn equal_range_by<'a, F>(&'a self, mut f: F) -> core::ops::Range<usize>
    where
        F: FnMut(&'a Self::Item) -> Ordering,
    {
        let s = self;
        let mut size = s.len();
        if size == 0 {
            return 0..0;
        }
        let mut base = (0usize, 0usize);
        while size > 1 {
            let half = size / 2;
            let mid = (base.0 + half, base.1 + half);
            let cmp = (
                f(unsafe { s.get_unchecked(mid.0) }),
                f(unsafe { s.get_unchecked(mid.1) }),
            );
            base = (
                if cmp.0 == Less { mid.0 } else { base.0 },
                if cmp.1 == Greater { base.1 } else { mid.1 },
            );
            size -= half;
        }
        let cmp = (
            f(unsafe { s.get_unchecked(base.0) }),
            f(unsafe { s.get_unchecked(base.1) }),
        );
        base.0 + (cmp.0 == Less) as usize..base.1 + (cmp.1 != Greater) as usize
    }

    fn equal_range_by_key<'a, K, F>(&'a self, k: &K, mut f: F) -> core::ops::Range<usize>
    where
        F: FnMut(&'a Self::Item) -> K,
        K: Ord,
    {
        self.equal_range_by(|e| f(e).cmp(k))
    }

    fn next_permutation(&mut self) -> bool
    where
        Self::Item: Ord
    {
        // Adapted from http://en.cppreference.com/w/cpp/algorithm/next_permutation.
        if self.len() <= 1 { return false; }
        let last = self.len() - 1;
        let mut a = last;
        loop {
            let mut b = a;
            a -= 1;
            if self[a] < self[b] {
                b = last;
                while self[a] >= self[b] {
                    b -= 1;
                }
                self.swap(a, b);
                self[a+1..].reverse();
                return true;
            }
            if a == 0 {
                self.reverse();
                return false;
            }
        }
    }

    fn prev_permutation(&mut self) -> bool
    where
        Self::Item: Ord
    {
        // Adapted from http://en.cppreference.com/w/cpp/algorithm/prev_permutation.
        if self.len() <= 1 { return false; }
        let last = self.len() - 1;
        let mut a = last;
        loop {
            let mut b = a;
            a -= 1;
            if self[b] < self[a] {
                b = last;
                while self[b] >= self[a] {
                    b -= 1;
                }
                self.swap(a, b);
                self[a+1..].reverse();
                return true;
            }
            if a == 0 {
                self.reverse();
                return false;
            }
        }
    }

    fn apply_permutation(&mut self, perm: &mut [isize]) {
        assert_eq!(self.len(), perm.len());
        assert!(self.len() < isize::max_value() as usize);
        for i in 0..self.len() as isize {
            let mut c = perm[i as usize];
            if c < 0 {
                perm[i as usize] = !c;
            } else if i != c {
                loop {
                    let n = perm[c as usize];
                    self.swap(c as usize, n as usize);
                    perm[c as usize] = !n;
                    c = n;
                    if i == c { break; }
                }
            }
        }
    }

    fn apply_inverse_permutation(&mut self, perm: &mut [isize]) {
        assert_eq!(self.len(), perm.len());
        assert!(self.len() < isize::max_value() as usize);
        for i in 0..self.len() as isize {
            let mut c = perm[i as usize];
            if c < 0 {
                perm[i as usize] = !c;
            } else if i != c {
                loop {
                    self.swap(c as usize, i as usize);
                    let n = perm[c as usize];
                    perm[c as usize] = !n;
                    c = n;
                    if i == c { break; }
                }
            }
        }
    }
}

pub trait Ext2 {
    /// Transforms the slice in the inverse permutation.
    /// 
    /// # Panics
    /// 
    /// This function panics if any value in `self` is not in `0..self.len()`.
    /// 
    /// # Example:
    /// 
    /// ```
    /// # use wt_slice::*;
    /// let mut p = [1, 3, 2, 0];
    /// p.invert_permutation();
    /// assert_eq!(p, [3, 0, 2, 1]);
    fn invert_permutation(&mut self);
}

impl Ext2 for [isize] {
    fn invert_permutation(&mut self) {
        assert!(self.len() < isize::max_value() as usize);
        for i in 0..self.len() as isize {
            let mut c = self[i as usize];
            if c < 0 {
                self[i as usize] = !c;
            } else if i != c {
                let mut n = i;
                loop {
                    let t = self[c as usize];
                    self[c as usize] = !n;
                    n = c;
                    c = t;
                    if c == i {
                        self[i as usize] = n;
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::{ExactSearch, Ext};

    #[test]
    fn exact_search_matches_insertion_semantics() {
        for size in 0..=128 {
            let values: std::vec::Vec<u32> = (0..size).map(|value| value * 2 + 2).collect();
            for needle in 0..=(size * 2 + 4) {
                let insertion = values.partition_point(|candidate| candidate < &needle);
                let expected = match values.get(insertion) {
                    Some(candidate) if candidate == &needle => Ok(insertion),
                    _ => Err(insertion),
                };

                assert_eq!(values.exact_search(&needle), expected, "size={size}, needle={needle}");
                assert_eq!(
                    values.exact_search_by(|candidate| candidate.cmp(&needle)),
                    expected,
                    "size={size}, needle={needle}",
                );
                assert_eq!(
                    values.exact_search_by_key(&needle, |candidate| *candidate),
                    expected,
                    "size={size}, needle={needle}",
                );
            }
        }
    }

    #[test]
    fn exact_search_accepts_any_equal_duplicate() {
        let values = [1, 3, 3, 3, 5];
        let found = values.exact_search(&3).unwrap();
        assert_eq!(values[found], 3);
    }

    #[test]
    fn lower_bound() {
        let b: [u32; 0] = [];
        assert_eq!(b.lower_bound(&0), 0);
        let b = [1, 3, 3, 5];
        assert_eq!(b.lower_bound(&0), 0);
        assert_eq!(b.lower_bound(&1), 0);
        assert_eq!(b.lower_bound(&2), 1);
        assert_eq!(b.lower_bound(&3), 1);
        assert_eq!(b.lower_bound(&4), 3);
        assert_eq!(b.lower_bound(&5), 3);
        assert_eq!(b.lower_bound(&6), 4);
    }

    #[test]
    fn upper_bound() {
        let b: [u32; 0] = [];
        assert_eq!(b.upper_bound(&0), 0);
        let b = [1, 3, 3, 5];
        assert_eq!(b.upper_bound(&0), 0);
        assert_eq!(b.upper_bound(&1), 1);
        assert_eq!(b.upper_bound(&2), 1);
        assert_eq!(b.upper_bound(&3), 3);
        assert_eq!(b.upper_bound(&4), 3);
        assert_eq!(b.upper_bound(&5), 4);
        assert_eq!(b.upper_bound(&6), 4);
    }

    #[test]
    fn equal_range() {
        let b: [u32; 0] = [];
        assert_eq!(b.equal_range(&0), 0..0);
        let b = [1, 3, 3, 5];
        assert_eq!(b.equal_range(&0), 0..0);
        assert_eq!(b.equal_range(&1), 0..1);
        assert_eq!(b.equal_range(&2), 1..1);
        assert_eq!(b.equal_range(&3), 1..3);
        assert_eq!(b.equal_range(&4), 3..3);
        assert_eq!(b.equal_range(&5), 3..4);
        assert_eq!(b.equal_range(&6), 4..4);
    }
}
