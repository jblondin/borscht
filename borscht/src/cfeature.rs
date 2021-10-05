/*!
 * Provies the primary cluster feature ([CFeature]) trait.
 */

use std::ops::Add;

use num_traits::Zero;

use crate::point::{Point, Scalar};

pub mod betula;
pub mod birch;

pub trait Dist<R> {
    fn dist2(&self, r: &R) -> Scalar;
    fn dist(&self, r: &R) -> Scalar {
        self.dist2(r).sqrt()
    }
}

pub enum Absorption<const DIMS: usize> {
    Absorbed,
    Failed(Point<DIMS>),
}

pub trait CFeature<const DIMS: usize>:
    Add<Self, Output = Self>
    + for<'a> Add<&'a Self, Output = Self>
    + Add<Point<DIMS>, Output = Self>
    + for<'a> Add<&'a Point<DIMS>, Output = Self>
    + Clone
    + Sized
    + Zero
    + Dist<Self>
    + Dist<Point<DIMS>>
    + From<Point<DIMS>>
{
    fn diam2(&self) -> Scalar;
    fn diam(&self) -> Scalar {
        self.diam2().sqrt()
    }
    fn center(&self) -> Point<DIMS>;
    fn size(&self) -> Scalar;
}
