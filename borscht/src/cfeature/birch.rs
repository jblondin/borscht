/*!
 * Standard cluster feature implementation.
 */

use std::ops::Add;

use serde::{Deserialize, Serialize};

use num_traits::Zero;

use crate::point::{Point, Scalar};

use super::Dist;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFeature<const DIMS: usize> {
    /// Linear Sum
    ls: Point<DIMS>,
    /// Sum of Squares
    ss: Scalar,
    /// Size
    n: usize,
}

impl<const DIMS: usize> Zero for CFeature<DIMS> {
    fn zero() -> CFeature<DIMS> {
        CFeature {
            ls: Point::zero(),
            ss: Scalar::zero(),
            n: usize::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.ls.is_zero() && self.ss.is_zero() && self.n.is_zero()
    }
}

impl<const DIMS: usize> Add<Self> for CFeature<DIMS> {
    type Output = CFeature<DIMS>;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(&rhs)
    }
}

impl<const DIMS: usize> Add<&Self> for CFeature<DIMS> {
    type Output = CFeature<DIMS>;

    fn add(self, rhs: &Self) -> Self::Output {
        CFeature {
            ls: self.ls + &rhs.ls,
            ss: self.ss + &rhs.ss,
            n: self.n + rhs.n,
        }
    }
}

impl<const DIMS: usize> Add<&Point<DIMS>> for CFeature<DIMS> {
    type Output = CFeature<DIMS>;

    fn add(self, rhs: &Point<DIMS>) -> Self::Output {
        CFeature {
            ls: self.ls + rhs,
            ss: self.ss + rhs.norm2(),
            n: self.n + 1,
        }
    }
}

impl<const DIMS: usize> Add<Point<DIMS>> for CFeature<DIMS> {
    type Output = CFeature<DIMS>;

    fn add(self, rhs: Point<DIMS>) -> Self::Output {
        self.add(&rhs)
    }
}

impl<const DIMS: usize> Dist<Point<DIMS>> for CFeature<DIMS> {
    fn dist2(&self, r: &Point<DIMS>) -> Scalar {
        (&self.ls - r).norm2()
    }
}

impl<const DIMS: usize> Dist<Self> for CFeature<DIMS> {
    fn dist2(&self, r: &Self) -> Scalar {
        (&self.ls - &r.ls).norm2()
    }
}

impl<const DIMS: usize> From<Point<DIMS>> for CFeature<DIMS> {
    fn from(orig: Point<DIMS>) -> CFeature<DIMS> {
        Self::zero() + orig
    }
}

impl<const DIMS: usize> crate::cfeature::CFeature<DIMS> for CFeature<DIMS> {
    fn diam2(&self) -> Scalar {
        (2.0 * self.n as Scalar * self.ss - 2.0 * self.ls.norm2())
            / if self.n < 2 {
                1 as Scalar
            } else {
                (self.n * (self.n - 1)) as Scalar
            }
    }
    fn size(&self) -> Scalar {
        self.n as Scalar
    }
    fn center(&self) -> Point<DIMS> {
        self.ls.clone() / (self.n as Scalar)
    }
}
