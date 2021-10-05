use std::{
    mem::MaybeUninit,
    ops::{Range, RangeInclusive},
};

use rand::{
    distributions::{uniform::SampleUniform, Distribution, Uniform},
    Rng,
};

pub struct MultivariateUniform<T: SampleUniform, const DIMS: usize> {
    inner: [Box<Uniform<T>>; DIMS],
}

macro_rules! impl_froms {
    ($($dim:expr)*) => {$(

impl<T: SampleUniform> From<[Range<T>; $dim]> for MultivariateUniform<T, $dim> {
    fn from(ranges: [Range<T>; $dim]) -> Self {
        MultivariateUniform {
            inner: {
                let mut data: [MaybeUninit<Box<Uniform<T>>>; $dim] =
                    unsafe { MaybeUninit::uninit().assume_init() };
                for (idx, range) in IntoIterator::into_iter(ranges).enumerate() {
                    data[idx] = MaybeUninit::new(Box::new(Uniform::from(range)));
                }
                unsafe { std::mem::transmute::<_, [Box<Uniform<T>>; $dim]>(data) }
            },
        }
    }
}
impl<T: SampleUniform> From<[RangeInclusive<T>; $dim]> for MultivariateUniform<T, $dim> {
    fn from(ranges: [RangeInclusive<T>; $dim]) -> Self {
        MultivariateUniform {
            inner: {
                let mut data: [MaybeUninit<Box<Uniform<T>>>; $dim] =
                    unsafe { MaybeUninit::uninit().assume_init() };
                for (idx, range) in IntoIterator::into_iter(ranges).enumerate() {
                    data[idx] = MaybeUninit::new(Box::new(Uniform::from(range)));
                }
                unsafe { std::mem::transmute::<_, [Box<Uniform<T>>; $dim]>(data) }
            },
        }
    }
}
impl<T: Sized + SampleUniform> Distribution<[T; $dim]> for MultivariateUniform<T, $dim> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [T; $dim] {
        let mut data: [MaybeUninit<T>; $dim] = unsafe { MaybeUninit::uninit().assume_init() };
        for (idx, dist) in self.inner.iter().enumerate() {
            data[idx] = MaybeUninit::new(dist.sample(rng));
        }
        unsafe {
            let out = std::ptr::read(&data as *const [MaybeUninit<T>; $dim] as *const [T; $dim]);
            std::mem::forget(data);
            out
        }
    }
}

    )*};
}
impl_froms!(
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

#[cfg(test)]
mod tests {
    use super::MultivariateUniform;
    use rand::distributions::Distribution;

    #[test]
    fn test_mv_uniform() {
        let bounds = ((0u8, 255u8), (0u8, 31u8), (128u8, 255u8));
        let dist = MultivariateUniform::from([
            bounds.0 .0..=bounds.0 .1,
            bounds.1 .0..=bounds.1 .1,
            bounds.2 .0..=bounds.2 .1,
        ]);
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let out = dist.sample(&mut rng);
            assert!(out[0] >= bounds.0 .0 && out[0] <= bounds.0 .1);
            assert!(out[1] >= bounds.1 .0 && out[1] <= bounds.1 .1);
            assert!(out[2] >= bounds.2 .0 && out[2] <= bounds.2 .1);
        }
    }
}
