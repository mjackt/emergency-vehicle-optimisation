mod vehicle;
mod types;
mod incident;

use std::collections::HashMap;
use std::fs;
use serde_json;

fn read_probs() -> HashMap<types::Location, u32> {
    // Attempt to read the JSON file
    let file_content = fs::read_to_string("map_data/probs.json").expect("Error loading probs.json");

    // Attempt to parse the JSON into a HashMap
    let raw_data: HashMap<String, u32> = serde_json::from_str(&file_content).expect("Error parsing probs.json");

    let mut data: HashMap<types::Location, u32> = HashMap::new();
    for (key, value) in raw_data {
        let key_as_u64 = key.parse::<u64>().expect("Invalid type in probs.json");
        data.insert(key_as_u64, value);
    }

    data
}

fn read_apsp() -> HashMap<types::Location, HashMap<types::Location, types::Time>> {
    // Attempt to read the JSON file
    let file_content = fs::read_to_string("map_data/apsp.json").expect("Error loading apsp.json");

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

fn main(){
    const TIMESTEP: types::Time = 300.0;

    let incident_probs: HashMap<types::Location, u32> = read_probs();

    let apsp: HashMap<types::Location, HashMap<types::Location, types::Time>> = read_apsp();



    let mut popo = vehicle::Vehicle::new(6728980081);
    let mut incidents: Vec<incident::Incident> = Vec::new();

    for i in 0..10{
    
        popo.timestep(TIMESTEP)
    }
}