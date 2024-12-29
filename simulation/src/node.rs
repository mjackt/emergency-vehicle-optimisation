use std::fmt;

use crate::types;

pub struct Node{
    out_locations: Vec<types::Location>,
    out_costs: Vec<types::Time>,
}

impl Node{
    pub fn new(out_locations: Vec<types::Location>, out_costs: Vec<types::Time>) -> Self{
        Self{
            out_locations,
            out_costs,
        }
    }

    pub fn get_out_locations(&self) -> &Vec<types::Location>{
        &self.out_locations
    }

    pub fn get_out_costs(&self) -> &Vec<types::Time>{
        &self.out_costs
    }
}

impl fmt::Display for Node{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.out_locations)
    }
}