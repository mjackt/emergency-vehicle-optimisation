mod vehicle;
mod types;
mod incident;
mod node;
mod dijkstra_node;
mod read_data;
mod simulation;
mod genetic;

use std::collections::HashMap;
use node::Node;
use rand::thread_rng;

use crate::genetic::{evolve_pop, avg_and_best_fitness};


fn main(){
    const TIMESTEP: types::Time = 300.0;
    const END_TIME: types::Time = 60.0 * 60.0 * 12.0;//12 hours

    let incident_probs: HashMap<types::Location, u32> = read_data::probs();

    println!("{}", incident_probs.len());

    let graph: HashMap<types::Location, Node> = read_data::graph();

    let base_locations: Vec<types::Location> = read_data::police();

    let mut route_cache: HashMap<(types::Location, types::Location), types::Time> = HashMap::new();

    let mut solutions : Vec<Vec<u8>> = Vec::new();

    let mut rng = thread_rng();

    const MAX_CARS: u8 = 8;

    const SOL_NUM: u8 = 40;

    const TIMEOUT: u8 = 20;

    for i in 0..SOL_NUM{
        solutions.push(genetic::generate_solution(base_locations.len() as u8, MAX_CARS, &mut rng));
    }
    println!("Solutions generated");

    avg_and_best_fitness(&solutions, 2, &incident_probs, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME);

    for i in 0..TIMEOUT{
        println!("{}/{}", i, TIMEOUT);
        evolve_pop(&mut solutions, SOL_NUM/2, SOL_NUM*3/4, 4, &incident_probs, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME, MAX_CARS, 1, &mut rng);
    }
    avg_and_best_fitness(&solutions, 2, &incident_probs, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME);
}