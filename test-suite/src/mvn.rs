use borscht::{
    cfeature::birch::CFeature as BirchFeature,
    cftree::{BasicConfig, BirchTree, Capacity, Node},
    point::Point,
};
use borscht_visualizer::{draw_to_file, VisualizerError};
use datagen::distribution::MultivariateNormal;

use rand::{distributions::Distribution, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;

pub type TreeNode = Node<BirchFeature<3>, 3>;

pub fn generate(seed: u64, count: usize) -> TreeNode {
    let means = [128u8, 52, 255];
    let stds = [5.0f64, 4.0f64, 3.0f64];
    let cov = [
        [stds[0] * stds[0], 0.0, 0.0],
        [0.0, stds[1] * stds[1], 0.0],
        [0.0, 0.0, stds[2] * stds[2]],
    ];
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
    let dist = MultivariateNormal::new(means, cov).expect("creation failure");
    let generator = std::iter::repeat_with(|| {
        let arr_u64 = dist.sample(&mut rng);
        Point::from_arr([arr_u64[0] as f64, arr_u64[1] as f64, arr_u64[2] as f64])
    });
    BirchTree::from_iter(
        generator.take(count),
        &BasicConfig {
            capacity: Capacity { min: 1, max: 3 },
            threshold: 0.5,
        },
    )
}

pub fn visualize(target_filename: &str, seed: u64, count: usize) -> Result<(), VisualizerError> {
    let tree = generate(seed, count);
    draw_to_file(target_filename, &tree)?;
    Ok(())
}
