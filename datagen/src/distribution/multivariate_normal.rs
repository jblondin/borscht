use num::traits::{bounds::Bounded, AsPrimitive};
use std::{marker::PhantomData, mem::MaybeUninit};
use thiserror::Error;

use nalgebra::{ArrayStorage, Cholesky, Const, Matrix, U1};
use rand::distributions::Distribution;
use rand_distr::StandardNormal;

#[derive(Error, Debug, PartialEq)]
pub enum MvnError {
    #[error("invalid (non-triangular) cov matrix")]
    NontriangularCov,
    #[error("cholesky decomposition failure")]
    CholeskyFailure,
    #[error("invalid values in mean array")]
    InvalidMeanValues,
    #[error("invalid values in cov array")]
    InvalidCovValues,
}

type Matrix64<const DIMS: usize> =
    Matrix<f64, Const<DIMS>, Const<DIMS>, ArrayStorage<f64, DIMS, DIMS>>;
type Vector64<const DIMS: usize> = Matrix<f64, Const<DIMS>, U1, ArrayStorage<f64, DIMS, 1>>;

#[derive(Debug, Clone)]
pub struct MultivariateNormal<T, const DIMS: usize> {
    mu: Vector64<DIMS>,
    chol_inv: Matrix64<DIMS>,
    chol_decomp: Matrix64<DIMS>,
    _marker: PhantomData<T>,
}

pub trait AllocMatrix<T> {
    type Input;
    fn alloc_matrix(input: Self::Input) -> Self;
}

pub trait AllocVector<T> {
    type Input;
    fn alloc_vector(input: Self::Input) -> Self;
}

macro_rules! impl_allocs {
    ($($dim:expr)*) => {$(

impl<T: AsPrimitive<f64>> AllocMatrix<T> for Matrix64<$dim> {
    type Input = [[T; $dim]; $dim];

    fn alloc_matrix(
        input: [[T; $dim]; $dim],
    ) -> Matrix64<$dim> {
        let mut matrix = unsafe { Matrix64::<$dim>::new_uninitialized() };
        for i in 0..$dim {
            for j in 0..$dim {
                unsafe {
                    (*matrix.as_mut_ptr())[(i, j)] = input[i][j].as_();
                }
            }
        }
        unsafe { std::mem::transmute::<_, Matrix64<$dim>>(matrix) }
    }
}

impl<T: AsPrimitive<f64>> AllocVector<T> for Vector64<$dim> {
    type Input = [T; $dim];

    fn alloc_vector(
        input: [T; $dim],
    ) -> Vector64<$dim> {
        let mut vector = unsafe { Vector64::<$dim>::new_uninitialized() };
        for i in 0..$dim {
            unsafe {
                (*vector.as_mut_ptr())[i] = input[i].as_();
            }
        }
        unsafe { std::mem::transmute::<_, Vector64<$dim>>(vector) }
    }
}

impl<T> Distribution<[T; $dim]> for MultivariateNormal<T, $dim>
where
    T: 'static + Copy + Bounded,
    f64: AsPrimitive<T>,
    T: AsPrimitive<f64>
{
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> [T; $dim] {
        let dist = StandardNormal;
        let mut zs = unsafe { Vector64::<$dim>::new_uninitialized() };
        let min_vals = Vector64::<$dim>::repeat(T::min_value().as_());
        let max_vals = Vector64::<$dim>::repeat(T::max_value().as_());
        let min_zs = &self.chol_inv * (&min_vals - &self.mu);
        let max_zs = &self.chol_inv * (&max_vals - &self.mu);
        for idx in 0..$dim {
            let valid_value =
                std::iter::repeat_with(|| -> f64 { dist.sample(rng) })
                .skip_while(|&x| x < min_zs[idx] || x > max_zs[idx])
                .next()
                .expect("'None' in a supposedly infinite iterator");
            unsafe {
                (*zs.as_mut_ptr())[idx] = valid_value;
            }
        }
        let zs = unsafe { std::mem::transmute::<_, Vector64<$dim>>(zs) };
        let data = (&self.chol_decomp * zs) + &self.mu;

        let mut maybe_out: [MaybeUninit<T>; $dim] = unsafe { MaybeUninit::uninit().assume_init() };
        for idx in 0..$dim {
            let val: T = data[idx].as_();
            maybe_out[idx] = MaybeUninit::new(val);
        }

        unsafe {
            let out = std::ptr::read(
                &maybe_out as *const [MaybeUninit<T>; $dim] as *const [T; $dim]
            );
            std::mem::forget(maybe_out);
            out
        }
    }
}

    )*};
}
impl_allocs!(
    1   2   3   4   5   6   7   8   9
10  11  12  13  14  15  16  17  18  19
20  21  22  23  24  25  26  27  28  29
30  31  32  33  34  35  36  37  38  39
40  41  42  43  44  45  46  47  48  49
50  51  52  53  54  55  56  57  58  59
60  61  62  63  64  65  66  67  68  69
70  71  72  73  74  75  76  77  78  79
80  81  82  83  84  85  86  87  88  89
90  91  92  93  94  95  96  97  98  99
100 101 102 103 104 105 106 107 108 109
110 111 112 113 114 115 116 117 118 119
120 121 122 123 124 125 126 127 128
);

impl<T, const DIMS: usize> MultivariateNormal<T, DIMS>
where
    Matrix64<DIMS>: AllocMatrix<f64, Input = [[f64; DIMS]; DIMS]>,
    Vector64<DIMS>: AllocVector<T, Input = [T; DIMS]>,
{
    pub fn new(mean: [T; DIMS], cov: [[f64; DIMS]; DIMS]) -> Result<Self, MvnError> {
        let mu = Vector64::alloc_vector(mean);
        let cov = Matrix64::alloc_matrix(cov);
        if mu.iter().any(|x| x.is_nan()) {
            return Err(MvnError::InvalidMeanValues);
        }
        if cov.iter().any(|x| x.is_nan()) {
            return Err(MvnError::InvalidCovValues);
        }
        if cov.upper_triangle() != cov.lower_triangle().transpose() {
            return Err(MvnError::NontriangularCov);
        }
        let chol = Cholesky::new(cov.clone()).ok_or(MvnError::CholeskyFailure)?;
        Ok(MultivariateNormal {
            mu,
            chol_inv: chol.inverse(),
            chol_decomp: chol.unpack(),
            _marker: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{MultivariateNormal, MvnError};
    use rand::distributions::Distribution;

    #[test]
    fn test_nan_cov() {
        let dist = MultivariateNormal::new(
            [128u8, 52, 255],
            [[20.0, 0.0, 0.0], [0.0, f64::NAN, 0.0], [0.0, 0.0, 5.0]],
        );
        assert!(dist.is_err());
        assert_eq!(dist.unwrap_err(), MvnError::InvalidCovValues);
    }

    #[test]
    fn test_chol_decomp_failure() {
        let dist = MultivariateNormal::new(
            [128u8, 52, 255],
            [[1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
        );
        assert!(dist.is_err());
        assert_eq!(dist.unwrap_err(), MvnError::CholeskyFailure);
    }

    #[test]
    fn test_nan_mean() {
        let dist = MultivariateNormal::new(
            [128.0f64, f64::NAN, 255.0],
            [[20.0, 0.0, 0.0], [0.0, 10.0, 0.0], [0.0, 0.0, 5.0]],
        );
        assert!(dist.is_err());
        assert_eq!(dist.unwrap_err(), MvnError::InvalidMeanValues);
    }

    #[test]
    fn test_nontriangular_cov() {
        let dist = MultivariateNormal::new(
            [128u8, 52, 255],
            [[20.0, 55.0, 0.0], [0.0, 10.0, 0.0], [0.0, 0.0, 5.0]],
        );
        assert!(dist.is_err());
        assert_eq!(dist.unwrap_err(), MvnError::NontriangularCov);
    }

    #[test]
    fn test_mv_normal() {
        let means = [128u8, 52, 255];
        let stds = [5.0f64, 4.0f64, 3.0f64];
        let cov = [
            [stds[0] * stds[0], 0.0, 0.0],
            [0.0, stds[1] * stds[1], 0.0],
            [0.0, 0.0, stds[2] * stds[2]],
        ];
        let dist = MultivariateNormal::new(means, cov).expect("creation failure");
        let mut rng = rand::thread_rng();
        const SIGMA_THRESHOLD: f64 = 6.0;
        const NUM_TESTS: usize = 100;
        for _ in 0..NUM_TESTS {
            let out = dist.sample(&mut rng);
            for i in 0..2 {
                assert!(
                    out[i] as f64 > means[i] as f64 - SIGMA_THRESHOLD * stds[i]
                        && (out[i] as f64) < means[i] as f64 + SIGMA_THRESHOLD * stds[i]
                );
            }
        }
    }
}
