/*!
 * Implementation for displaying the structure of a constructure CFTree.
 */

use crate::{
    cfeature::CFeature,
    cftree::{Node, NodeEntry},
};

pub trait DisplayTree {
    fn display_tree_at_level(&self, level: usize, max_depth: Option<usize>);
    fn display_tree(&self) {
        self.display_tree_at_level(0, None)
    }
    fn display_tree_to_depth(&self, depth: usize) {
        self.display_tree_at_level(0, Some(depth))
    }
}

impl<CF: CFeature<DIMS>, const DIMS: usize> DisplayTree for Node<CF, DIMS> {
    fn display_tree_at_level(&self, level: usize, max_depth: Option<usize>) {
        self.entries.iter().for_each(|entry| {
            entry.display_tree_at_level(level, max_depth);
        })
    }
}

impl<CF: CFeature<DIMS>, const DIMS: usize> DisplayTree for NodeEntry<CF, DIMS> {
    fn display_tree_at_level(&self, level: usize, max_depth: Option<usize>) {
        if max_depth.map_or(false, |d| level >= d) {
            return;
        }
        for _ in 0..level {
            print!("-");
        }
        println!(
            "Feature center={:?} diam={} size={}",
            self.feature.center(),
            self.feature.diam(),
            self.feature.size()
        );
        if let Some(ref child) = self.child {
            child.display_tree_at_level(level + 1, max_depth);
        }
    }
}
