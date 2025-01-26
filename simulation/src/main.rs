mod vehicle;
mod types;
mod incident;
mod node;
mod dijkstra_node;
mod read_data;
mod simulation;
mod genetic;
mod data;

use std::collections::HashMap;
use data::Data;
use incident::Incident;
use node::Node;
use rand::thread_rng;

use crate::genetic::{evolve_pop, avg_and_best_fitness};
use crate::simulation::generate_incidents;

fn main(){

    let incident_probs: HashMap<types::Location, u32> = read_data::probs();

    println!("{}", incident_probs.len());

    let graph: HashMap<types::Location, Node> = read_data::graph();

    let base_locations: Vec<types::Location> = read_data::police();

    let mut route_cache: HashMap<(types::Location, types::Location), types::Time> = HashMap::new();

    let mut solutions : Vec<Vec<u8>> = Vec::new();

    let mut rng = thread_rng();

    //TUNABLES
    //Sim stuff
    const MAX_CARS: u16 = 120;
    const TIMESTEP: types::Time = 300.0;
    const END_TIME: types::Time = 60.0 * 60.0 * 12.0;//Secs. Not inclusive i.e when time hits end time its over
    const PROBABILITY_WEIGHTING: f64 = 0.4;
    //GA stuff
    const EVAL_ITER: u8 = 1;//Should always be 1 if using incident plan
    const SOL_NUM: u16 = 200;//Must be div by 4
    const TIMEOUT: u16 = 30;

    let total_steps: f32 = END_TIME/TIMESTEP;

    let mut spawn_stack: Vec<Vec<incident::Incident>> = Vec::new();
    let mut spawn_time: types::Time = 0.0;
    for _ in 0..total_steps as usize{
        let mut step_incidents: Vec<Incident> = Vec::new();
        generate_incidents(&mut step_incidents, &incident_probs, TIMESTEP, spawn_time, PROBABILITY_WEIGHTING, &mut rng);
        spawn_stack.push(step_incidents);
        spawn_time += TIMESTEP;
    }
    println!("Incident plan generated");

    for _ in 0..SOL_NUM{
        solutions.push(genetic::generate_solution(base_locations.len() as u8, MAX_CARS, &mut rng));
    }
    println!("Solutions generated");

    let start: Data = avg_and_best_fitness(&solutions, 1, &mut spawn_stack, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME);

    for i in 0..TIMEOUT{
        println!("{}/{}", i, TIMEOUT);
        evolve_pop(&mut solutions, SOL_NUM/2, SOL_NUM*3/4, EVAL_ITER, &spawn_stack, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME, MAX_CARS, 1, &mut rng);
    }
    let end: Data = avg_and_best_fitness(&solutions, 1, &mut spawn_stack, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME);

    println!("*************\nStarting solutions:\n{}", start);

    println!("\n\nFinal solutions:\n{}\n", end);

    let names: Vec<String> = read_data::police_names();
    let solution: &Vec<u8> = end.get_best_solution();
    for i in 0..solution.len(){
        println!("{}: {}", names[i], solution[i]);
    }
}