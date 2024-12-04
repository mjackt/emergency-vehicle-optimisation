use std::collections::HashMap;
use std::fs;
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read the JSON file
    let file_content = fs::read_to_string("map_data/probs.json")?;
    
    // Parse the JSON into a HashMap
    let data: HashMap<String, i32> = serde_json::from_str(&file_content)?;

    // Print the HashMap
    for (key, value) in &data {
        println!("Key: {}, Value: {}", key, value);
    }

    Ok(())
}

