use crate::types;
use std::cmp::Ordering;

#[derive(Debug, PartialEq)]
pub struct DijkstraNode {
    pub cost: types::Time,
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