use borscht::{
    cfeature::birch::CFeature as BirchFeature,
    cftree::{BasicConfig, BirchTree, Capacity, Node},
    point::Point,
};
use borscht_visualizer::{draw_to_file, VisualizerError};

pub type TreeNode = Node<BirchFeature<3>, 3>;

pub fn generate(_seed: u64) -> TreeNode {
    let mut points = vec![
        Point::from_arr([1.0, 2.0, 3.0]),
        Point::from_arr([2.0, 2.0, 3.0]),
        Point::from_arr([1.0, 3.0, 3.0]),
        Point::from_arr([1.0, 2.0, 4.0]),
    ];
    BirchTree::from_iter(
        points.drain(..),
        &BasicConfig {
            capacity: Capacity { min: 1, max: 3 },
            threshold: 0.5,
        },
    )
}

pub fn visualize(target_filename: &str) -> Result<(), VisualizerError> {
    let tree = generate(0);
    draw_to_file(target_filename, &tree)?;
    Ok(())
}
