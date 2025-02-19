//! Module containing all methods related to the police response simulation.
//! 
//! The simulation can be explained as such:
//! 1. Spawn new incidents using the `spawn_stack` incident schedule
//! 2. Dispatch vehicles to incidents as efficiently as possible
//! 3. Step the simulation and vehciles using the desired timstep
//! 4. If the time < end_time:
//!     Go back to the start
//!    Else:
//!     Sum and return performance metrics
//! 
//! The simulation has been designed to run as efficiently as possible by using serveral concepts:
//! ##### Hash Maps
//! Due to the large amount of items and info being stored, hash maps are used pretty much everywhere to achieve O(1) accessing.
//! ##### Route Caching
//! Route calculations are done using Dijkstra and all knowledge gained from a Dijkstra run is cached in a HashMap.
//! Before performing any calculations, this cache is checked to see if the calculation has been performed before and therefore doesn't need to be done again.
//! ##### Incident Plan
//! Predetermining the incident plan provides a fair test for all solutions. However it also has a couple performance boosts:
//! - Makes route caching insanely powerful. If the incidents are always in the same place then journeys will be similar accross all solutions. Therefore caching is utilised to the maximum.
//! - Removes need to randomly generate new incidents for every simulation.
//! ##### Travel simplification
//! A vehicle will always store the cost of it's latest journey. Then if a vehcile has just left base and now wants to return to base, it will just use this last travel cost value for the reverse journey rather than recalculating the new cost.
use crate::incident::Incident;
use crate::types;
use crate::dijkstra_node::DijkstraNode;
use crate::node::Node;
use crate::vehicle;

use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::collections::HashSet;
use rand::rngs::ThreadRng;
use rand::Rng;
use weighted_rand::table::WalkerTable;

/// Adds new calculation knowledge into the route cache
/// ### Parameters
/// * `route_cache` - The current hash map of knowledge.
/// * `new_knowledge` - A hashmap of time calculations.
/// * `source` - The source node where all calculations begin.
fn update_route_cache(route_cache: &mut HashMap<(types::Location, types::Location), types::Time>, new_knowledge: &HashMap<types::Location, types::Time>, source: types::Location){
    for (key, value) in new_knowledge{
        route_cache.insert((source, *key), *value);
    }
}

/// Performs a dijkstra algorithm. Terminates after target is located.
/// ### Parameters
/// * `graph` - The road network graph as a HashMap.
/// * `route_cache` - The current hash map of knowledge.
/// * `source` - The starting node for the Dijkstra search.
/// * `target` - The target node for the Dijkstra path.
/// ### Returns
/// * `Option<types::Time>` - An option either containing the time from source to target or None if there is no path.
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
    local_discovery.insert(target, f32::MAX);//Marks as unreachable
    update_route_cache(route_cache, &local_discovery, source);
    None
}

/// Calculates travel time from source to target or uses cache.
/// ### Parameters
/// * `source` - The starting node for the Dijkstra search.
/// * `target` - The target node for the Dijkstra path.
/// * `route_cache` - The current hash map of knowledge.
/// * `graph` - The road network graph as a HashMap.
/// ### Returns
/// * `Option<types::Time>` - An option either containing the time from source to target or None if there is no path.
fn calc_travel_time(source: types::Location, target: types::Location, route_cache: &mut HashMap<(types::Location, types::Location), types::Time>, graph: &HashMap<types::Location, Node>) -> Option<types::Time>{
    match route_cache.get(&(source, target)){
        Some(time) => {
            if *time == f32::MAX{//Already known to be unreachable
                return None
            }
            return Some(*time)
        }
        None => {
            return dijkstra(graph, route_cache, source, target);
        }
    }
}

/// Dispatches vehicles to active incidents.
/// ### Parameters
/// * `vehicles` - Vec of all Vehicle objects.
/// * `incidents` - The Vec of current incidents.
/// * `graph` - The road network graph as a HashMap.
/// * `route_cache` - The current hash map of knowledge.
/// * `current_time` - The current time of the simulation
fn dispatching(vehicles: &mut Vec<vehicle::Vehicle>,
                incidents: &mut Vec<Incident>,
                graph: &HashMap<types::Location, Node>,
                route_cache: &mut HashMap<(types::Location, types::Location), types::Time>,
                current_time: types::Time){
            
    for event in incidents{
        if event.is_solved() || !event.is_valid(){
            continue;
        }
        let vehicles_required: usize = event.get_vehicles_required();
        let mut closest_times: Vec<types::Time> = vec![std::f32::MAX; vehicles_required]; 
        let mut closest_cars: Vec<&mut vehicle::Vehicle> = Vec::new();
        let target: types::Location = event.get_location();

        for car in &mut *vehicles{
            if car.get_secs_till_free() == 0.0{
                let current_loc: types::Location = car.get_location();
                match calc_travel_time(current_loc, target, route_cache, graph){
                    Some(travel_time) => {
                        for i in 0..closest_times.len(){
                            if travel_time < closest_times[i]{
                                closest_times.insert(i, travel_time);
                                closest_cars.insert(i, car);
                                break
                            }
                        }
                        //Restore to correct lengths
                        while closest_times.len() > vehicles_required{
                            closest_times.pop();
                        }
                        while closest_cars.len() > vehicles_required{
                            closest_cars.pop();
                        }
                    }
                    None => {//Graph must be disconnected
                        event.unreachable();
                        break;
                    }
                }
            }
        }

        let incident_time: types::Time = event.get_service_time();
        for i in 0..closest_cars.len(){
            closest_cars[i].goto(target,closest_times[i], incident_time);
        }
        event.solved(current_time + closest_times[vehicles_required - 1]);//Event is solved after last car arrives
        //println!("{}: {} responding to incident at {}. Response time: {}\n", current_time, closest_car.get_name(), target, closest_time);
    }

    for car in vehicles{
        if car.get_secs_till_free() == 0.0 && car.get_location() != car.get_base(){
            if car.last_move_from_base() == true{//If it just came from base and now needs to return to base. We can just reuse calculated travel time for the reverse journey
                car.goto(car.get_base(), car.get_last_travel(), 0.0);
                continue
            }
            match calc_travel_time(car.get_location(), car.get_base(), route_cache, graph){
                Some(travel_time) => {
                    car.goto(car.get_base(), travel_time, 0.0);
                }
                None => {
                    //Should never really occur but beause osm boundaries are weird sometimes there may be large unescapable areas
                    //Just resets car with previous travel cost
                    car.reset();
                }
            }

            //println!("{}: {} returning to base. Travel time: {}\n", current_time, car.get_name(), travel_time);
        }
    }
}

/// Generates incidents based on probabilistic occurrence rates.
/// ### Parameters
/// * `incidents` - A mutable reference to a `Vec<Incident>`.
/// * `incident_probs` - A `HashMap` mapping locations to probabilities representing the chance of an incident occurring at that location per timestep.
/// * `timestep` - The simulation timestep.
/// * `current_time` - The current simulation time.
/// * `probability_weighting` - A weight applied to incident probability.
/// * `rngthread` - A mutable reference to a random number generator.
/// * `wa_table` - A reference to a `WalkerTable`, which is used to determine vehicle allocation.
/// * `vehicle_count` - A mutable reference to an array of vehicle counts, tracking the number of incdients with each number of vehciles.
/// * `service_time_mean` - The mean service time for an incident.
/// * `_service_time_std` - The standard deviation of the service time (currently unused).
pub fn generate_incidents(incidents: &mut Vec<Incident>, incident_probs: &HashMap<types::Location, f32>, timestep: types::Time, current_time: types::Time, probability_weighting: f64, rngthread: &mut ThreadRng, wa_table: &WalkerTable, vehicle_count: &mut [u32; 5], service_time_mean: f32, _service_time_std: f32){
    for (location, num_per_year) in incident_probs{
        //Could save some maths here
        let prob: f64 = *num_per_year as f64 / 365.0 / 24.0 / 60.0 / 60.0 * timestep as f64 * probability_weighting;//Converting num per year into probabilty per timestep
        let ran_float: f64 = rngthread.random();
        if ran_float < prob{
            let vehicle_num = wa_table.next() + 1;
            vehicle_count[vehicle_num-1] += 1;
            let service_time: types::Time = service_time_mean;
            /*let service_time: types::Time = rng().sample::<f32,_>(Normal::new(service_time_mean, service_time_std).unwrap());
            if service_time < 0.0{
                panic!("Negative service time");//Obvs don't have to panic here. Could just set it to 0. Feels like it would be good to be aware tho.
            }*/

            let new_incident: Incident = Incident::new(*location, service_time, current_time, vehicle_num);
            incidents.push(new_incident);
        }
    }
}

/// Adds incidents to the list of active ones. They are spawned from the spawn stack.
/// ### Parameters
/// * `spawn_stack` - Reference to a 2D Vec containing incidents to spawn at each timestep.
/// * `incidents` - A mutable reference to a `Vec<Incident>`.
/// * `index` - The index of spawn stack to use.
fn spawn_incidents(spawn_stack: &Vec<Vec<Incident>>, incidents: &mut Vec<Incident>, index: usize){
    let mut this_step_incidents: Vec<Incident> = spawn_stack[index].clone();
    incidents.append(&mut this_step_incidents);
}

/// Completes a timestep on each vehicle
/// ### Parameters
/// * `vehicles` - Vec of all Vehicle objects.
/// * `timestep` - The simulation timestep.
fn step_vehicles(vehicles: &mut Vec<vehicle::Vehicle>, timestep: types::Time){
    for vehicle in vehicles{
        vehicle.timestep(timestep);
    }
}

/// Runs the simulation once.
/// ### Parameters
/// * `graph` - The road network graph as a HashMap.
/// * `spawn_stack` - Reference to a 2D Vec containing incidents to spawn at each timestep.
/// * `vehicles` - Vec of all Vehicle objects.
/// * `route_cache` - The current hash map of knowledge.
/// * `timestep` - The simulation timestep.
/// * `end_time` - The time to end simulation at.
/// * `unreachable_set` - A HashSet of locations known to be unreachable that will be updated throughout the simulation.
/// ### Returns
/// * `types::Time` - The average response time.
fn run(graph: &HashMap<types::Location, Node>, spawn_stack: &Vec<Vec<Incident>>, vehicles: &mut Vec<vehicle::Vehicle>, route_cache: &mut HashMap<(types::Location, types::Location), types::Time>, timestep: types::Time, end_time: types::Time, unreachable_set: &mut HashSet<types::Location>) -> types::Time{
    let mut incidents: Vec<Incident> = Vec::new();
    let mut time: types::Time = 0.0;

    while time < end_time{
        spawn_incidents(spawn_stack, &mut incidents, (time/timestep) as usize);
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
        else{
            unreachable_set.insert(event.get_location());
        }
    }

    let avg_response_time: types::Time = sum_response_times / inc_count as f32;
    //println!("Responded to {} incidents", inc_count);
    return avg_response_time
}

/// Evaluates a solution.
/// ### Parameters
/// * `solution` - The solution to be evaluated.
/// * `iterations` - The number of iterations for the solution to be evaluated. Redundant should always be one currently.
/// * `spawn_stack` - Reference to a 2D Vec containing incidents to spawn at each timestep.
/// * `graph` - The road network graph as a HashMap.
/// * `base_locations` - A Vec of locations in which police vehicles can be based at.
/// * `route_cache` - The current hash map of knowledge.
/// * `timestep` - The simulation timestep.
/// * `end_time` - The time to end simulation at.
/// * `unreachable_set` - A HashSet of locations known to be unreachable that will be updated throughout the simulation.
/// ### Returns
/// * `types::Time` - The average average response time.
pub fn evaluate(solution: &Vec<u8>,
            iterations: u8,
            spawn_stack: &Vec<Vec<Incident>>,
            graph: &HashMap<types::Location, Node>,
            base_locations: &Vec<types::Location>,
            route_cache: &mut HashMap<(types::Location, types::Location), types::Time>,
            timestep: types::Time,
            end_time: types::Time,
            unreachable_set: &mut HashSet<types::Location>,
            ) -> types::Time{
    let mut sum_avg_response_times: types::Time = 0.0;

    for _ in 0..iterations{
        let mut vehicles: Vec<vehicle::Vehicle> = Vec::new();

        let mut counter: u8 = 0;
        let mut i: usize = 0;
        for base in base_locations{
            for _ in 0..solution[i]{
                let name: String = format!("Car {}", counter);
                let car: vehicle::Vehicle = vehicle::Vehicle::new(*base, name);
                vehicles.push(car);
                counter += 1;
            }
            i += 1;
        }

        let avg_response_time: types::Time = run(graph, spawn_stack, &mut vehicles, route_cache, timestep, end_time, unreachable_set);

        sum_avg_response_times += avg_response_time;
    }
    let solutions_avg_response: types::Time = sum_avg_response_times / iterations as f32;
    solutions_avg_response
}
