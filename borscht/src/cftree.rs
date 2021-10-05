/*!
 * Cluster Feature tree struct and implementation.
 */

use std::{collections::HashSet, fmt::Debug};

use serde::{Deserialize, Serialize};

use itertools::{Either, Itertools};
use num_traits::Float;

use crate::{
    cfeature::{betula::CFeature as BetulaFeature, birch::CFeature as BirchFeature, CFeature},
    point::{Point, Scalar},
};

#[derive(Debug, Clone)]
pub struct Capacity {
    pub min: usize,
    pub max: usize,
}

pub trait TreeConfig {
    fn node_capacity(&self) -> &Capacity;
    fn leaf_capacity(&self) -> &Capacity {
        self.node_capacity()
    }
    fn threshold(&self) -> Scalar;
}

#[derive(Debug, Clone)]
pub struct BasicConfig {
    pub capacity: Capacity,
    pub threshold: Scalar,
}
impl TreeConfig for BasicConfig {
    fn node_capacity(&self) -> &Capacity {
        &self.capacity
    }
    fn threshold(&self) -> Scalar {
        self.threshold
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Node<CF, const DIMS: usize> {
    pub entries: Vec<NodeEntry<CF, DIMS>>,
}

impl<CF: CFeature<DIMS>, const DIMS: usize> Node<CF, DIMS> {
    pub fn new<'a, TC: TreeConfig>(config: &'a TC) -> Node<CF, DIMS> {
        Node {
            entries: Vec::with_capacity(config.node_capacity().max + 1),
        }
    }

    pub fn with_entries(entries: Vec<NodeEntry<CF, DIMS>>) -> Node<CF, DIMS> {
        Node { entries }
    }

    pub fn height(&self) -> usize {
        1 + self
            .entries
            .iter()
            .map(|entry| entry.height())
            .max()
            .unwrap_or(0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeEntry<CF, const DIMS: usize> {
    pub feature: CF,
    pub child: Option<Node<CF, DIMS>>,
}

impl<CF: CFeature<DIMS>, const DIMS: usize> Default for NodeEntry<CF, DIMS> {
    fn default() -> NodeEntry<CF, DIMS> {
        NodeEntry {
            feature: CF::zero(),
            child: None,
        }
    }
}

impl<'a, CF: CFeature<DIMS>, const DIMS: usize> NodeEntry<CF, DIMS> {
    fn with_point(orig: Point<DIMS>) -> NodeEntry<CF, DIMS> {
        NodeEntry {
            feature: CF::from(orig),
            child: None,
        }
    }
    fn height(&self) -> usize {
        self.child.as_ref().map(|node| node.height()).unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub enum EntryInsertion<Point> {
    Success,
    Failure(Point),
}

impl<CF: CFeature<DIMS>, const DIMS: usize> NodeEntry<CF, DIMS> {
    fn insert<'a, TC: TreeConfig>(
        &mut self,
        p: Point<DIMS>,
        config: &'a TC,
    ) -> EntryInsertion<Point<DIMS>> {
        // check if feature can absorb point
        let feature_with_point = self.feature.clone() + &p;
        match feature_with_point.diam2() <= config.threshold() {
            true => {
                self.feature = feature_with_point;
                EntryInsertion::Success
            }
            false => EntryInsertion::Failure(p),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NodeInsertion<T> {
    Single(T),
    Split(T, T),
}

impl<CF, const DIMS: usize> Node<CF, DIMS>
where
    CF: CFeature<DIMS> + Debug + Clone,
{
    fn compute_feature(&self) -> CF {
        self.entries
            .iter()
            .map(|entry| &entry.feature)
            .fold(CF::zero(), |acc, feature| acc + feature)
    }
    fn check_split<'a, TC: TreeConfig>(mut self, config: &'a TC) -> NodeInsertion<Self> {
        match self.entries.len() >= config.node_capacity().max {
            true => {
                // time to split!
                // find farthest
                let Farthest {
                    lidx,
                    ridx,
                    mut rest,
                    ..
                } = self.farthest();
                // assign entries to respective closest
                let lset = rest
                    .drain()
                    .filter(|&idx| {
                        self.entries[lidx].feature.dist2(&self.entries[idx].feature)
                            < self.entries[ridx].feature.dist2(&self.entries[idx].feature)
                    })
                    .collect::<HashSet<_>>();
                // return split
                let (left, right) =
                    self.entries
                        .drain(..)
                        .enumerate()
                        .partition_map(|(idx, entry)| match lset.contains(&idx) {
                            true => Either::Left(entry),
                            false => Either::Right(entry),
                        });

                NodeInsertion::Split(Node::with_entries(left), Node::with_entries(right))
            }
            _ => NodeInsertion::Single(self),
        }
    }

    fn insert<'a, TC: TreeConfig>(mut self, p: Point<DIMS>, config: &'a TC) -> NodeInsertion<Self> {
        // find closest cluster
        match self
            .entries
            .iter_mut()
            .fold((None, Scalar::max_value()), |(_, closest_dist2), entry| {
                let d2 = entry.feature.dist2(&p);
                match d2 < closest_dist2 {
                    true => (Some(entry), d2),
                    false => (None, closest_dist2),
                }
            })
            .0
        {
            Some(entry) if entry.child.is_some() => {
                let child_node = entry.child.as_mut().unwrap();
                let mut temp_node = Node::new(config);
                // make empty node the temporary child of this entry
                std::mem::swap(child_node, &mut temp_node);
                // insert into previous child node
                match temp_node.insert(p, config) {
                    NodeInsertion::Split(mut left, right) => {
                        // put the 'left' into the previous spot where child was
                        std::mem::swap(child_node, &mut left);
                        // update computed features of left
                        entry.feature = child_node.compute_feature();
                        // add new entry with 'right'
                        self.entries.push(NodeEntry {
                            feature: right.compute_feature(),
                            child: Some(right),
                        });
                        self.check_split(config)
                    }
                    NodeInsertion::Single(mut node) => {
                        std::mem::swap(child_node, &mut node);
                        // update computed features of node
                        entry.feature = child_node.compute_feature();
                        NodeInsertion::Single(self)
                    }
                }
            }
            Some(entry) => match entry.insert(p, config) {
                EntryInsertion::Success => NodeInsertion::Single(self),
                EntryInsertion::Failure(p) => {
                    self.entries.push(NodeEntry::with_point(p));
                    self.check_split(config)
                }
            },
            None => {
                self.entries.push(NodeEntry::with_point(p));
                NodeInsertion::Single(self)
            }
        }
    }

    pub fn from_iter<'a, T: IntoIterator<Item = Point<DIMS>>, TC: TreeConfig>(
        iter: T,
        config: &'a TC,
    ) -> Self {
        let mut root = Node::new(config);
        for (_i, p) in iter.into_iter().enumerate() {
            match root.insert(p, config) {
                NodeInsertion::Single(node) => {
                    root = node;
                }
                NodeInsertion::Split(left, right) => {
                    root = Node {
                        entries: vec![
                            NodeEntry {
                                feature: left.compute_feature(),
                                child: Some(left),
                            },
                            NodeEntry {
                                feature: right.compute_feature(),
                                child: Some(right),
                            },
                        ],
                    };
                }
            }
            // root.display_tree();
        }
        root
    }
}

struct Farthest {
    farthest_dist2: Scalar,
    lidx: usize,
    ridx: usize,
    rest: HashSet<usize>,
}

impl<CF, const DIMS: usize> Node<CF, DIMS>
where
    CF: CFeature<DIMS> + Debug + Clone,
{
    fn farthest<'b>(&'b self) -> Farthest {
        let pairwise_iter = self.entries.iter().enumerate().tuple_combinations();

        let tracker = pairwise_iter
            .fold(
                None,
                |tracker: Option<Farthest>, ((lidx, lnode), (ridx, rnode))| {
                    let dist2 = lnode.feature.dist2(&rnode.feature);
                    Some(match tracker {
                        Some(mut t) if dist2 > t.farthest_dist2 => {
                            // switched farthest nodes
                            t.rest.insert(t.lidx);
                            t.rest.insert(t.ridx);
                            Farthest {
                                farthest_dist2: dist2,
                                lidx,
                                ridx,
                                rest: t.rest,
                            }
                        }
                        Some(mut t) => {
                            // no change to farthest nodes
                            t.rest.insert(lidx);
                            t.rest.insert(ridx);
                            t
                        }
                        // initial
                        None => Farthest {
                            farthest_dist2: dist2,
                            lidx,
                            ridx,
                            rest: HashSet::new(),
                        },
                    })
                },
            )
            .expect("unexpected empty node");
        tracker
    }
}

pub type BirchTree<const DIMS: usize> = Node<BirchFeature<DIMS>, DIMS>;
pub type BetulaTree<const DIMS: usize> = Node<BetulaFeature<DIMS>, DIMS>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_arr() {
        let mut points = vec![
            Point::from_arr([1.0, 2.0, 3.0]),
            Point::from_arr([2.0, 2.0, 3.0]),
            Point::from_arr([1.0, 3.0, 3.0]),
            Point::from_arr([1.0, 2.0, 4.0]),
        ];
        let root = BirchTree::from_iter(
            points.drain(..),
            &BasicConfig {
                capacity: Capacity { min: 1, max: 3 },
                threshold: 0.5,
            },
        );
        println!("{:#?}", root);
    }
}
