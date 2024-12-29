mod vehicle;
mod types;
mod incident;
mod node;
mod dijkstra_node;

use dijkstra_node::DijkstraNode;
use std::collections::HashMap;
use std::fs;
use std::collections::BinaryHeap;
use std::panic::Location;
use node::Node;
use serde_json;
use rand::Rng;

const PLACE: &str = "exeter";

fn read_probs() -> HashMap<types::Location, u32> {
    // Attempt to read the JSON file
    let path = format!("map_data/{}/probs.json", PLACE);
    let file_content = fs::read_to_string(path).expect("Error loading probs.json");

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

fn read_graph() -> HashMap<types::Location, Node>{
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

fn read_police() -> Vec<types::Location>{
    let path = format!("map_data/{}/police.json", PLACE);
    let file_content = fs::read_to_string(path).expect("Error loading police.json");

    let data: Vec<types::Location> = serde_json::from_str(&file_content).expect("Error parsing police.json");

    data
}

fn update_route_cache(route_cache: &mut HashMap<(types::Location, types::Location), types::Time>, new_knowledge: &HashMap<types::Location, types::Time>, source: types::Location){
    for (key, value) in new_knowledge{
        route_cache.insert((source, *key), *value);
    }
}

fn dijkstra(graph: &HashMap<types::Location, Node>, route_cache: &mut HashMap<(types::Location, types::Location), types::Time>, source: types::Location, target: types::Location) -> Option<types::Time>{
    let mut heap = BinaryHeap::new();
    let mut local_discovery: HashMap<types::Location, types::Time>= HashMap::new();

    heap.push(DijkstraNode{cost: 0.0, location: source});

    while let Some(DijkstraNode{cost, location}) = heap.pop(){
        if location == target{
            local_discovery.insert(target, cost);
            update_route_cache(route_cache, &local_discovery, source);
            return Some(cost)
        }

        match local_discovery.get(&location){
            None => {},
            Some(prev_found_cost) => {
                if cost > *prev_found_cost{
                    continue;
                }
            }
        }

        let current_node: &Node = graph.get(&location).unwrap();

        for i in 0..current_node.get_out_locations().len(){
            let out: types::Location = current_node.get_out_locations()[i];
            let total_cost: types::Time = cost + current_node.get_out_costs()[i];
            match local_discovery.get(&out){
                Some(prev_found_cost) => {
                    if *prev_found_cost > total_cost{
                        local_discovery.insert(out, total_cost);
                        heap.push(DijkstraNode{cost: total_cost, location: out})
                    }
                    else{
                        continue
                    }
                }
                None => {
                    local_discovery.insert( out, total_cost);
                    heap.push(DijkstraNode{cost: total_cost, location: out})
                }
            }
        }
    }
    update_route_cache(route_cache, &local_discovery, source);
    None
}

fn calc_travel_time(source: types::Location, target: types::Location, route_cache: &mut HashMap<(types::Location, types::Location), types::Time>, graph: &HashMap<types::Location, Node>) -> Option<types::Time>{
    match route_cache.get(&(source, target)){
        Some(time) => return Some(time.clone()),
        None => {
            return dijkstra(graph, route_cache, source, target);
        }
    }
}

fn dispatching(vehicles: &mut Vec<vehicle::Vehicle>,
                incidents: &mut Vec<incident::Incident>,
                graph: &HashMap<types::Location, Node>,
                route_cache: &mut HashMap<(types::Location, types::Location), types::Time>,
                current_time: types::Time){
            
    for event in incidents{
        if event.is_solved(){
            continue;
        }

        let mut closest_time: types::Time = std::f32::MAX; 
        let mut closest_car: Option<&mut vehicle::Vehicle> = None;

        let target: types::Location = event.get_location();

        for car in &mut *vehicles{
            if car.get_secs_till_free() == 0.0{
                let current_loc: types::Location = car.get_location();
                match calc_travel_time(current_loc, target, route_cache, graph){
                    Some(travel_time) => {
                        if travel_time < closest_time{
                            closest_time = travel_time;
                            closest_car = Some(car);
                        }
                    }
                    None => {//Graph must be disconnected
                        println!("Unreachable {}", event.get_location());
                        event.unreachable();
                        continue;
                    }
                }
            }
        }
        match closest_car{
            Some(closest_car) => {
                let incident_time: types::Time = event.get_service_time();

                closest_car.goto(target, closest_time, incident_time);
                event.solved(current_time + closest_time);
                println!("{}: {} responding to incident at {}. Response time: {}\n", current_time, closest_car.get_name(), target, closest_time);
            },
            None => (),
        }
    }

    for car in vehicles{
        if car.get_secs_till_free() == 0.0 && car.get_location() != car.get_base(){
            let travel_time: types::Time = calc_travel_time(car.get_location(), car.get_base(), route_cache, graph).unwrap();

            car.goto(car.get_base(), travel_time, 0.0);
            println!("{}: {} returning to base. Travel time: {}\n", current_time, car.get_name(), travel_time);
        }
    }
}

fn generate_incidents(incidents: &mut Vec<incident::Incident>, incident_probs: &HashMap<types::Location, u32>, timestep: types::Time, current_time: types::Time, probability_weighting: f64){
    let mut rng = rand::thread_rng();
    for (location, num_per_year) in incident_probs{
        //Could save some maths here
        let prob: f64 = *num_per_year as f64 / 365.0 / 24.0 / 60.0 / 60.0 * timestep as f64 * probability_weighting;//Converting num per year into probabilty per timestep

        let ran_float: f64 = rng.gen();
        if ran_float < prob{
            let service_time: types::Time = rand::thread_rng().gen_range(600..2400) as f32;

            let new_incident: incident::Incident = incident::Incident::new(*location, service_time, current_time);
            incidents.push(new_incident);
        }
    }
}

fn step_vehicles(vehicles: &mut Vec<vehicle::Vehicle>, timestep: types::Time){
    for vehicle in vehicles{
        vehicle.timestep(timestep);
    }
}

fn simulation(graph: &HashMap<types::Location, Node>, incident_probs: &HashMap<types::Location, u32>, vehicles: &mut Vec<vehicle::Vehicle>, route_cache: &mut HashMap<(types::Location, types::Location), types::Time>, timestep: types::Time, end_time: types::Time) -> types::Time{
    const PROBABILTY_WEIGHTING: f64 = 1.5;
    let mut incidents: Vec<incident::Incident> = Vec::new();
    let mut time: types::Time = 0.0;

    while time < end_time{
        generate_incidents(&mut incidents, &incident_probs, timestep, time, PROBABILTY_WEIGHTING);
        dispatching(vehicles, &mut incidents, graph, route_cache, time);
        
        step_vehicles(vehicles, timestep);

        time = time + timestep;
    }

    let mut sum_response_times: types::Time = 0.0;
    let mut inc_count: usize = 0;

    for event in incidents{
        if event.is_valid(){
            inc_count += 1;
            match event.get_resolved_time(){
                Some(resolved_time) => {
                    let response_time: types::Time = resolved_time - event.get_creation_time();
                    sum_response_times += response_time;
                },
                None => {
                    let response_time: types::Time = time - event.get_creation_time();
                    sum_response_times += response_time;
                }
            }
        }
    }

    let avg_response_time: types::Time = sum_response_times / inc_count as f32;

    return avg_response_time
}

fn main(){
    const TIMESTEP: types::Time = 300.0;
    const END_TIME: types::Time = 60.0 * 60.0 * 12.0;//12 hours

    let incident_probs: HashMap<types::Location, u32> = read_probs();
    
    //let apsp: HashMap<types::Location, HashMap<types::Location, types::Time>> = read_apsp();

    let graph: HashMap<types::Location, Node> = read_graph();

    let mut vehicles: Vec<vehicle::Vehicle> = Vec::new();
    let base_locations: Vec<types::Location> = read_police();

    let mut route_cache: HashMap<(types::Location, types::Location), types::Time> = HashMap::new();

    let mut solution: Vec<u8> = vec![0; base_locations.len()];

    solution[0] = 2;
    solution[1] = 2;

    let mut counter: u8 = 0;
    let mut i: usize = 0;
    for base in base_locations{
        for _ in 0..solution[i]{
            let name: String = format!("Car {}", counter);
            let car: vehicle::Vehicle = vehicle::Vehicle::new(base, name);
            vehicles.push(car);
            counter += 1;
        }
        i += 1;
    }

    for i in 0..5{
        let avg_response_time: types::Time = simulation(&graph, &incident_probs, &mut vehicles, &mut route_cache, TIMESTEP, END_TIME);

        println!("{}", avg_response_time);
    }
}