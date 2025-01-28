//! Module containing the DijkstraNode struct and it's implementations to provide comparisons.
use crate::types;
use std::cmp::Ordering;

///A struct to store Dijkstra calculations for a Node.
#[derive(Debug, PartialEq)]
pub struct DijkstraNode {
    ///The cost to reach the node.
    pub cost: types::Time,
    ///The location of the node. Corresponds to OSM id.
    pub location: types::Location,
}

impl Eq for DijkstraNode {}

impl Ord for DijkstraNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse the ordering so the smallest priority becomes the top of the heap
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

// `PartialOrd` not implmented by default for floats for god knows why
impl PartialOrd for DijkstraNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}