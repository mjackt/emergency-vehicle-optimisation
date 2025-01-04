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

use crate::genetic::{tournament_selection, crossover, evolve_pop, avg_and_best_fitness};


fn main(){
    const TIMESTEP: types::Time = 300.0;
    const END_TIME: types::Time = 60.0 * 60.0 * 12.0;//12 hours

    let incident_probs: HashMap<types::Location, u32> = read_data::probs();

    let graph: HashMap<types::Location, Node> = read_data::graph();

    let base_locations: Vec<types::Location> = read_data::police();

    let mut route_cache: HashMap<(types::Location, types::Location), types::Time> = HashMap::new();

    let mut solutions : Vec<Vec<u8>> = Vec::new();

    for i in 0..12{
        solutions.push(genetic::generate_solution(3, 5));
    }

    avg_and_best_fitness(&solutions, 2, &incident_probs, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME);

    for i in 0..10{
        println!("{:?}", solutions);
        evolve_pop(&mut solutions, 6, 8, 2, &incident_probs, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME, 5, 1);
    }
    avg_and_best_fitness(&solutions, 2, &incident_probs, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME);
}