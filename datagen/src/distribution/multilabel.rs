use rand::{distributions::Distribution, Rng};

pub struct MultiLabelDistribution<Dist> {
    dists: Vec<Dist>,
}

impl<Dist> MultiLabelDistribution<Dist> {
    pub fn new_single(dist: Dist) -> Self {
        MultiLabelDistribution { dists: vec![dist] }
    }
    pub fn new(dists: Vec<Dist>) -> Self {
        MultiLabelDistribution { dists }
    }
    fn choose<Prng: Rng + ?Sized>(&self, rng: &mut Prng) -> &Dist {
        let n = self.dists.len();
        &self.dists[rng.gen_range(0..n)]
    }
}

impl<T: Sized, Dist: Distribution<[T; 3]>> Distribution<[T; 3]> for MultiLabelDistribution<Dist> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> [T; 3] {
        self.choose(rng).sample(rng)
    }
}
