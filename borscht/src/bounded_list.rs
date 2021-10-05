/*!
 * A bounded-length list implementation. Provides [BoundedList], a structure that can contain a
 * number of element between prespecified minimum and maximum length.
 *
 * TODO: I wrote this because I thought I wanted it, but then never used it. I should probably
 * move it into a small standalone crate just in case it ever comes up useful anywhere.
 */

use std::fmt::Debug;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
#[error("invalid bounds")]
pub struct InvalidBounds;

#[derive(Error, Debug, PartialEq)]
#[error("minimum bound exceeded")]
pub struct MinBoundExceeded;

#[derive(Error, Debug, PartialEq)]
#[error("maximum bound exceeded")]
pub struct MaxBoundExceeded<T: Debug>(pub T);

#[derive(Debug, Clone, PartialEq)]
pub struct BoundedList<T> {
    values: Vec<T>,
    min: usize,
    max: usize,
}

impl<T> BoundedList<T> {
    fn check_bounds(len: usize, min: usize, max: usize) -> Result<(), InvalidBounds> {
        match (min > max, len < min, len > max) {
            (false, false, false) => Ok(()),
            _ => Err(InvalidBounds),
        }
    }

    pub fn with_max(max: usize) -> BoundedList<T> {
        BoundedList {
            values: vec![],
            min: 0,
            max,
        }
    }

    pub fn from_arr<const N: usize>(
        initial_values: [T; N],
        min: usize,
        max: usize,
    ) -> Result<BoundedList<T>, InvalidBounds> {
        Self::check_bounds(N, min, max)?;
        Ok(BoundedList {
            values: Vec::from(initial_values),
            min,
            max,
        })
    }

    pub fn from_iter<I: IntoIterator<Item = T>>(
        initial_values: I,
        min: usize,
        max: usize,
    ) -> Result<BoundedList<T>, InvalidBounds> {
        let values = initial_values.into_iter().collect::<Vec<T>>();
        Self::check_bounds(values.len(), min, max)?;
        Ok(BoundedList { values, min, max })
    }

    pub fn push(&mut self, item: T) -> Result<(), MaxBoundExceeded<T>>
    where
        T: Debug,
    {
        if self.values.len() >= self.max {
            return Err(MaxBoundExceeded(item));
        }
        self.values.push(item);
        return Ok(());
    }

    pub fn pop(&mut self) -> Result<T, MinBoundExceeded> {
        if self.values.len() <= self.min {
            return Err(MinBoundExceeded);
        }
        Ok(self.values.pop().expect("impossible empty values array"))
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            iter: self.values.iter(),
        }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        IterMut {
            inner: self.values.iter_mut(),
        }
    }

    pub fn drain<'a>(&'a mut self) -> DrainIter<'a, T> {
        DrainIter {
            inner: self.values.drain(..),
        }
    }

    pub fn max_size(&self) -> usize {
        self.max
    }

    pub fn min_size(&self) -> usize {
        self.min
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

#[derive(Debug)]
pub struct Iter<'a, T: 'a> {
    iter: std::slice::Iter<'a, T>,
}

impl<'a, T: 'a> Clone for Iter<'a, T> {
    fn clone(&self) -> Self {
        Iter {
            iter: self.iter.clone(),
        }
    }
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct IterMut<'a, T> {
    inner: std::slice::IterMut<'a, T>,
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub struct DrainIter<'a, T: 'a> {
    inner: std::vec::Drain<'a, T>,
}

impl<'a, T: 'a> Iterator for DrainIter<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_arr() {
        let list_result = BoundedList::from_arr([4, 2, 5, 5], 4, 3);
        assert!(list_result.is_err());
        assert_eq!(list_result.unwrap_err(), InvalidBounds);

        let list_result = BoundedList::from_arr([4, 2, 5, 5], 4, 4);
        assert!(list_result.is_ok());
        assert_eq!(
            list_result.unwrap(),
            BoundedList {
                values: vec![4, 2, 5, 5],
                min: 4,
                max: 4
            }
        );

        let list_result = BoundedList::from_arr([4, 2, 5, 5], 2, 4);
        assert!(list_result.is_ok());
        assert_eq!(
            list_result.unwrap(),
            BoundedList {
                values: vec![4, 2, 5, 5],
                min: 2,
                max: 4
            }
        );

        let list_result = BoundedList::from_arr([4, 2, 5], 2, 4);
        assert!(list_result.is_ok());
        assert_eq!(
            list_result.unwrap(),
            BoundedList {
                values: vec![4, 2, 5],
                min: 2,
                max: 4
            }
        );

        let list_result = BoundedList::from_arr([4, 2], 2, 4);
        assert!(list_result.is_ok());
        assert_eq!(
            list_result.unwrap(),
            BoundedList {
                values: vec![4, 2],
                min: 2,
                max: 4
            }
        );

        let list_result = BoundedList::from_arr([4], 2, 4);
        assert!(list_result.is_err());
        assert_eq!(list_result.unwrap_err(), InvalidBounds);

        let list_result = BoundedList::from_arr([4, 2, 5, 5, 1], 2, 4);
        assert!(list_result.is_err());
        assert_eq!(list_result.unwrap_err(), InvalidBounds);
    }

    #[test]
    fn from_iter() {
        let list_result = BoundedList::from_iter(vec![4, 2, 5, 5], 4, 4);
        assert!(list_result.is_ok());
        assert_eq!(
            list_result.unwrap(),
            BoundedList {
                values: vec![4, 2, 5, 5],
                min: 4,
                max: 4
            }
        );

        let list_result = BoundedList::from_iter(vec![4, 2, 5], 4, 4);
        assert!(list_result.is_err());
        assert_eq!(list_result.unwrap_err(), InvalidBounds);

        let list_result = BoundedList::from_iter(vec![4, 2, 5, 5, 1], 2, 4);
        assert!(list_result.is_err());
        assert_eq!(list_result.unwrap_err(), InvalidBounds);
    }
}
