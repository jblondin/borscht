/*!
 * Betula cluster feature implementation.
 */

use std::ops::Add;

use num_traits::Zero;
use serde::{Deserialize, Serialize};

use crate::point::{Point, Scalar};

use super::Dist;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CFeature<const DIMS: usize> {
    /// Sum of weights
    n: Scalar,
    /// Weighted mean
    mu: Point<DIMS>,
    /// Weighted sum of squared deviations from mean
    s: Point<DIMS>,
}

impl<const DIMS: usize> Zero for CFeature<DIMS> {
    fn zero() -> CFeature<DIMS> {
        CFeature {
            n: Scalar::zero(),
            mu: Point::zero(),
            s: Point::zero(),
        }
    }

    fn is_zero(&self) -> bool {
        self.n.is_zero() && self.mu.is_zero() && self.s.is_zero()
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
        let n = self.n + rhs.n;
        let mu = &self.mu + rhs.n / n * (&rhs.mu - &self.mu);
        CFeature {
            n,
            mu: mu.clone(),
            s: self.s + &rhs.s + rhs.n * (&self.mu - &rhs.mu) * (mu - &rhs.mu),
        }
    }
}

impl<const DIMS: usize> Add<&Point<DIMS>> for CFeature<DIMS> {
    type Output = CFeature<DIMS>;

    fn add(self, rhs: &Point<DIMS>) -> Self::Output {
        self + CFeature {
            n: 1.0,
            mu: rhs.clone(),
            s: Point::<DIMS>::zero(),
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
        (&self.mu - r).norm2()
    }
}

impl<const DIMS: usize> Dist<Self> for CFeature<DIMS> {
    fn dist2(&self, r: &Self) -> Scalar {
        (&self.mu - &r.mu).norm2()
    }
}

impl<const DIMS: usize> From<Point<DIMS>> for CFeature<DIMS> {
    fn from(orig: Point<DIMS>) -> CFeature<DIMS> {
        Self::zero() + orig
    }
}

impl<const DIMS: usize> crate::cfeature::CFeature<DIMS> for CFeature<DIMS> {
    fn diam2(&self) -> Scalar {
        2.0 / self.n * self.s.norm2()
    }
    fn size(&self) -> Scalar {
        self.n
    }
    fn center(&self) -> Point<DIMS> {
        self.mu.clone()
    }
}
