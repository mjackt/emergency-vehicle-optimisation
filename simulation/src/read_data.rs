//! Module contains methods focused on reading the input files required to run the simulation and GA.
//! 
//! This includes:
//! - probs.json
//! - police_ids.json
//! - police_names.json
//! - graph.json
use std::{collections::HashMap, fs};

use crate::types;
use crate::node::Node;

const PLACE: &str = "dnc_12months_50agg";

/// Returns a HashMap of locations and incident probabilites using the *probs.json* file.
/// ### Returns
/// `HashMap<types::Location, f32>` - A HashMap keyed by location ID which an associated value which represents how many incidents will occur at that location over a year.
pub fn probs() -> HashMap<types::Location, f32> {
    // Attempt to read the JSON file
    let path = format!("map_data/{}/probs.json", PLACE);
    let file_content = fs::read_to_string(path).expect("Error loading probs.json");

    // Attempt to parse the JSON into a HashMap
    let raw_data: HashMap<String, f32> = serde_json::from_str(&file_content).expect("Error parsing probs.json");

    let mut data: HashMap<types::Location, f32> = HashMap::new();
    for (key, value) in raw_data {
        if value == 0.0{
            continue
        }
        let key_as_u64 = key.parse::<u64>().expect("Invalid type in probs.json");
        data.insert(key_as_u64, value);
    }

    data
}

///OBSOLETE
fn apsp() -> HashMap<types::Location, HashMap<types::Location, types::Time>> {
    // Attempt to read the JSON file
    let path = format!("map_data/{}/apsp.json", PLACE);
    let file_content = fs::read_to_string(path).expect("Error loading apsp.json");

    // Attempt to parse the JSON into a HashMap
    let raw_data: HashMap<String, HashMap<String, types::Time>> = serde_json::from_str(&file_content).expect("Error parsing apsp.json");

    let mut data: HashMap<types::Location, HashMap<types::Location, types::Time>> = HashMap::new();
    for (key, raw_value) in raw_data {
        let key_as_u64 = key.parse::<u64>().expect("Invalid type in apsp.json");

        let mut value: HashMap<types::Location, types::Time> = HashMap::new();

        for (inner_key, float) in raw_value {
            let inner_key_as_u64 = inner_key.parse::<u64>().expect("Invalid type in apsp.json");

            value.insert(inner_key_as_u64, float);
    
        data.insert(key_as_u64, value.clone());
        }
    }

    data
}

/// Returns a HashMap of location IDs and Node objects using the *graph.json* file.
/// ### Returns
/// `HashMap<types::Location, Node>` - A HashMap keyed by location ID with the values as [Node] objects.
pub fn graph() -> HashMap<types::Location, Node>{
    let path = format!("map_data/{}/graph.json", PLACE);
    let file_content = fs::read_to_string(path).expect("Error loading graph.json");
    let raw_data: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(&file_content).expect("Error parsing graph.json");

    let mut graph: HashMap<types::Location, Node> = HashMap::new();

    for (key, value) in raw_data{
        let value_as_obj = value.as_object().unwrap();

        let outs = value_as_obj.get("outs").unwrap().as_array().unwrap();
        let mut out_costs: Vec<types::Time> = Vec::new();
        let mut out_locations: Vec<types::Location> = Vec::new();

        for out in outs{
            let out_id: types::Location = out.get("id").unwrap().as_u64().unwrap();
            let out_cost: types::Time = out.get("cost").unwrap().as_f64().unwrap() as f32;
            out_locations.push(out_id);
            out_costs.push(out_cost)
        }
        let new_obj: Node = Node::new(out_locations, out_costs);

        let id: types::Location = key.parse().unwrap();

        graph.insert(id, new_obj);
    }

    graph
}

/// Returns a list of all node IDs that are a police base. Uses police_ids.json.
/// ### Returns
/// `Vec<types::Location>` - A list of all node IDs that are a police base.
pub fn police() -> Vec<types::Location>{
    let path = format!("map_data/{}/police_ids.json", PLACE);
    let file_content = fs::read_to_string(path).expect("Error loading police_ids.json");

    let data: Vec<types::Location> = serde_json::from_str(&file_content).expect("Error parsing police_ids.json");

    data
}

/// Returns a list of the names of every police base. Uses police_names.json.
/// ### Returns
/// `Vec<String>` - A list of all police base names.
pub fn police_names() -> Vec<String>{
    let path = format!("map_data/{}/police_names.json", PLACE);
    let file_content = fs::read_to_string(path).expect("Error loading police_names.json");

    let data: Vec<String> = serde_json::from_str(&file_content).expect("Error parsing police_names.json");
    data
}