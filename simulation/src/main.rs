mod vehicle;
mod types;
mod incident;

use std::collections::HashMap;
use std::fs;
use incident::Incident;
use serde_json;
use rand::Rng;

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

fn dispatching(vehicles: &mut Vec<vehicle::Vehicle>,
                incidents: &mut Vec<incident::Incident>,
                apsp: &HashMap<types::Location, HashMap<types::Location, types::Time>>,
                current_time: types::Time){
            
    for event in incidents{
        if event.is_solved(){
            continue;
        }
        if vehicles[0].get_secs_till_free() == 0.0{
            let current_loc: types::Location = vehicles[0].get_location();
            let target: types::Location = event.get_location();
            let travel_time: types::Time = *apsp.get(&current_loc).unwrap().get(&target).unwrap();
            let incident_time: types::Time = event.get_service_time();

            vehicles[0].goto(target, travel_time, incident_time);
            event.solved(current_time + travel_time);
        }
    }
}

fn generate_incidents(incidents: &mut Vec<incident::Incident>, incident_probs: &HashMap<types::Location, u32>, timestep: types::Time, current_time: types::Time){
    let mut rng = rand::thread_rng();
    for (location, num_per_year) in incident_probs{
        //Could save some maths here
        let prob: f64 = *num_per_year as f64 / 365.0 / 24.0 / 60.0 / 60.0 * timestep as f64;//Converting num per year into probabilty per timestep


        let ran_float: f64 = rng.gen();
        if ran_float < prob{
            let service_time: types::Time = rand::thread_rng().gen_range(600..2400) as f32;

            let new_incident: Incident = Incident::new(*location, service_time, current_time);
            incidents.push(new_incident);
        }
    }
}

fn main(){
    const TIMESTEP: types::Time = 300.0;
    const END_TIME: types::Time = 60.0 * 60.0 * 100.0;//12 hours

    let incident_probs: HashMap<types::Location, u32> = read_probs();
    
    let apsp: HashMap<types::Location, HashMap<types::Location, types::Time>> = read_apsp();

    let mut time: types::Time = 0.0;

    let popo = vehicle::Vehicle::new(489738904);
    let mut incidents: Vec<incident::Incident> = Vec::new();
    let mut vehicles: Vec<vehicle::Vehicle> = Vec::new();

    vehicles.push(popo);

    while time < END_TIME{
        println!("{}\n", vehicles[0]);

        generate_incidents(&mut incidents, &incident_probs, TIMESTEP, time);
        dispatching(&mut vehicles, &mut incidents, &apsp, time);
        
        vehicles[0].timestep(TIMESTEP);
        time = time + TIMESTEP;
        println!("{}", time);
    }

    let mut sum_response_times: types::Time = 0.0;
    let mut inc_count: usize = 0;

    for event in incidents{
        inc_count += 1;
        match event.get_resolved_time(){
            Some(resolved_time) => {
                let response_time: types::Time = resolved_time - event.get_creation_time();
                println!("{}", response_time);
                sum_response_times += response_time;
            },
            None => {
                let response_time: types::Time = time - event.get_creation_time();
                sum_response_times += response_time;
            }
        }
    }

    let avg_response_time: types::Time = sum_response_times / inc_count as f32;

    println!("Average response time: {}", avg_response_time);
}