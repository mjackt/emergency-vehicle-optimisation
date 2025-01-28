//! Module containg the [Node] struct and it's associated methods.
use std::fmt;

use crate::types;

/// Represents a node on the graph.
/// 
/// Note: a node doesn't store it's own location as they are stored in a HashMap where the key is it's [Location].
pub struct Node{
    /// The IDs of nodes that can be reached by leaving this node.
    out_locations: Vec<types::Location>,
    /// The associated cost of travelling to each location in `out_locations`.
    /// 
    /// Index 0 corresponds to the cost of going to index 0 in `out_locations` and so on.
    out_costs: Vec<types::Time>,
}

impl Node{
    ///Returns a new Node object
    pub fn new(out_locations: Vec<types::Location>, out_costs: Vec<types::Time>) -> Self{
        Self{
            out_locations,
            out_costs,
        }
    }
    ///Returns a reference to the list of out IDs
    pub fn get_out_locations(&self) -> &Vec<types::Location>{
        &self.out_locations
    }
    ///Returns a reference to the list containing out costs.
    pub fn get_out_costs(&self) -> &Vec<types::Time>{
        &self.out_costs
    }
}

impl fmt::Display for Node{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.out_locations)
    }
}