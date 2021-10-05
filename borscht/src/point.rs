/*!
 * Main data point structure and associated trait implementations.
 */

use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Rem, RemAssign, Sub,
    SubAssign,
};

use num_traits::Zero;
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};

pub type Scalar = f64;

#[derive(Debug, Clone, PartialEq)]
pub struct Point<const DIMS: usize>([Scalar; DIMS]);

impl<const DIMS: usize> Serialize for Point<DIMS> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_tuple(DIMS)?;
        for e in &self.0 {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}

struct PointVisitor<const DIMS: usize>;

impl<'de, const DIMS: usize> Visitor<'de> for PointVisitor<DIMS> {
    // type Value = [T; DIMS];
    type Value = Point<DIMS>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_fmt(format_args!("a point of dimensionality {}", DIMS))
    }

    #[inline]
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut p = Point::default();
        for i in 0..DIMS {
            match seq.next_element()? {
                Some(val) => p.0[i] = val,
                None => return Err(serde::de::Error::invalid_length(i, &self)),
            }
        }
        Ok(p)
    }
}

impl<'de, const DIMS: usize> Deserialize<'de> for Point<DIMS> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_tuple(DIMS, PointVisitor::<DIMS>)
    }
}

impl<const DIMS: usize> Point<DIMS> {
    pub fn from_arr(arr: [Scalar; DIMS]) -> Point<DIMS> {
        Point(arr)
    }
    pub fn as_slice(&self) -> &[Scalar] {
        &self.0
    }
    pub fn as_mut_slice(&mut self) -> &mut [Scalar] {
        &mut self.0
    }
    pub fn norm2(&self) -> Scalar {
        self.0.iter().fold(Scalar::default(), |acc, x| acc + x * x)
    }
}

impl<const DIMS: usize> Zero for Point<DIMS> {
    fn zero() -> Point<DIMS> {
        Point([Scalar::zero(); DIMS])
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(|x| x.is_zero())
    }
}

impl<const DIMS: usize> Default for Point<DIMS> {
    fn default() -> Point<DIMS> {
        Self::zero()
    }
}

impl<I, const DIMS: usize> Index<I> for Point<DIMS>
where
    [Scalar]: Index<I>,
{
    type Output = <[Scalar] as Index<I>>::Output;
    fn index(&self, index: I) -> &Self::Output {
        self.0.index(index)
    }
}

impl<I, const DIMS: usize> IndexMut<I> for Point<DIMS>
where
    [Scalar]: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.0.index_mut(index)
    }
}

macro_rules! impl_op {
    ($op_trait:ident $fname:ident $op:tt $op_assign:tt $op_assign_trait:ident $fname_assign:ident) => {
        mod $fname {
            use super::Scalar;
            pub fn assign_l<const DIMS: usize>(left: &mut [Scalar], right: &[Scalar]) {
                for i in 0..DIMS {
                    left[i] $op_assign right[i]
                }
            }
            pub fn assign_l_scalar<const DIMS: usize>(left: &mut [Scalar], right: Scalar) {
                for i in 0..DIMS {
                    left[i] $op_assign right
                }
            }
            pub fn assign_r<const DIMS: usize>(left: &[Scalar], right: &mut [Scalar]) {
                for i in 0..DIMS {
                    right[i] = left[i] $op right[i]
                }
            }
        }

        impl<const DIMS: usize> $op_trait<Point<DIMS>> for Point<DIMS> {
            type Output = Point<DIMS>;

            fn $fname(mut self, rhs: Point<DIMS>) -> Self::Output {
                $fname::assign_l::<DIMS>(self.as_mut_slice(), rhs.as_slice());
                self
            }
        }

        impl<const DIMS: usize> $op_trait<&Point<DIMS>> for Point<DIMS> {
            type Output = Point<DIMS>;

            fn $fname(mut self, rhs: &Point<DIMS>) -> Self::Output {
                $fname::assign_l::<DIMS>(self.as_mut_slice(), rhs.as_slice());
                self
            }
        }

        impl<const DIMS: usize> $op_trait<Point<DIMS>> for &Point<DIMS> {
            type Output = Point<DIMS>;

            fn $fname(self, mut rhs: Point<DIMS>) -> Self::Output {
                $fname::assign_r::<DIMS>(self.as_slice(), rhs.as_mut_slice());
                rhs
            }
        }

        impl<const DIMS: usize> $op_trait<&Point<DIMS>> for &Point<DIMS> {
            type Output = Point<DIMS>;

            fn $fname(self, rhs: &Point<DIMS>) -> Self::Output {
                let mut out = self.clone();
                $fname::assign_l::<DIMS>(out.as_mut_slice(), rhs.as_slice());
                out
            }
        }

        impl<const DIMS: usize> $op_assign_trait<Point<DIMS>> for Point<DIMS> {
            fn $fname_assign(&mut self, rhs: Point<DIMS>) {
                $fname::assign_l::<DIMS>(self.as_mut_slice(), rhs.as_slice());
            }
        }

        impl<const DIMS: usize> $op_assign_trait<&Point<DIMS>> for Point<DIMS> {
            fn $fname_assign(&mut self, rhs: &Point<DIMS>) {
                $fname::assign_l::<DIMS>(self.as_mut_slice(), rhs.as_slice());
            }
        }

        impl<const DIMS: usize> $op_trait<Scalar> for Point<DIMS> {
            type Output = Point<DIMS>;

            fn $fname(mut self, rhs: Scalar) -> Self::Output {
                $fname::assign_l_scalar::<DIMS>(self.as_mut_slice(), rhs);
                self
            }
        }

        impl<const DIMS: usize> $op_trait<Scalar> for &Point<DIMS> {
            type Output = Point<DIMS>;

            fn $fname(self, rhs: Scalar) -> Self::Output {
                let mut ret = self.clone();
                $fname::assign_l_scalar::<DIMS>(ret.as_mut_slice(), rhs);
                ret
            }
        }

        impl<const DIMS: usize> $op_trait<Point<DIMS>> for Scalar {
            type Output = Point<DIMS>;

            fn $fname(self, mut rhs: Point<DIMS>) -> Self::Output {
                $fname::assign_l_scalar::<DIMS>(rhs.as_mut_slice(), self);
                rhs
            }
        }

        impl<const DIMS: usize> $op_trait<&Point<DIMS>> for Scalar {
            type Output = Point<DIMS>;

            fn $fname(self, rhs: &Point<DIMS>) -> Self::Output {
                let mut ret = rhs.clone();
                $fname::assign_l_scalar::<DIMS>(ret.as_mut_slice(), self);
                ret
            }
        }
    };
}
impl_op!(Add add + += AddAssign add_assign);
impl_op!(Sub sub - -= SubAssign sub_assign);
impl_op!(Mul mul * *= MulAssign mul_assign);
impl_op!(Div div / /= DivAssign div_assign);
impl_op!(Rem rem % %= RemAssign rem_assign);

impl<const DIMS: usize> Neg for Point<DIMS> {
    type Output = Point<DIMS>;
    fn neg(mut self) -> Self::Output {
        for i in 0..DIMS {
            self.0[i] = -self.0[i]
        }
        self
    }
}
impl<const DIMS: usize> Neg for &Point<DIMS> {
    type Output = Point<DIMS>;
    fn neg(self) -> Self::Output {
        let out = self.clone();
        -out
    }
}
