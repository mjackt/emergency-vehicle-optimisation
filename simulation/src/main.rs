//! # What is it?
//! A program to optimise the locations of police vehicles.
//! 
//! The program will take input files, which allow you to customise the area and police stations used for the experiment.
//! It will then generate and improve solutions using a GA paired with a response simulation, which will provide an average response time to be used as a fitness function.
//! An incident schedule is generated at the start of the program and every solution is tested on the same schedule to ensure fairness. This means one single run of this program will not provide a solution best to all circumstances.
//! 
//! # Tuning GA
//! A variety of paremeters are available to tune the GA.
//! Some relate to the GA process, i.e tournament size or mutation count.
//! Others refer to the simulation, i.e timestep or the max number of cars available.
//! 
//! By default they are set to the parameters I have found most succesfull. However, this will change based on the area and constraints being used.
//! 
//! # Set Up
//! Requires 3 files to be in place:
//! - probs.json
//! - police_ids.json
//! - police_names.json
//! - graph.json
//! 
//! These files should be placed in a folder named x. Where x is the customisable variable 'PLACE' in read_data.rs.
//! 
//! To generate the files you must run the python script: map_data.py
mod vehicle;
mod types;
mod incident;
mod node;
mod dijkstra_node;
mod read_data;
mod simulation;
mod genetic;
mod data;

use std::collections::{HashMap, HashSet};
use std::time::Instant;
use data::Data;
use incident::Incident;
use node::Node;
use rand::rngs::ThreadRng;
use rand::thread_rng;
use types::Solution;

use crate::genetic::{evolve_pop, avg_and_best_fitness};
use crate::simulation::generate_incidents;

/// Runs the entire GA process on input data pointed to by PLACE in read_data.rs.
/// 
/// After execution it will provide info on the GA process such as the final best solution, change in solution average and timing info.
fn main(){

    let incident_probs: HashMap<types::Location, f32> = read_data::probs();

    println!("{} nodes have more than 0 incidents per year", incident_probs.len());

    let graph: HashMap<types::Location, Node> = read_data::graph();

    let base_locations: Vec<types::Location> = read_data::police();

    let mut route_cache: HashMap<(types::Location, types::Location), types::Time> = HashMap::new();

    let mut solutions : Vec<Solution> = Vec::new();

    let mut rng: ThreadRng = thread_rng();

    //TUNABLES
    //Sim stuff
    const MAX_CARS: u16 = 50;
    const TIMESTEP: types::Time = 120.0;
    const END_TIME: types::Time = 60.0 * 60.0 * 12.0;//Secs. Not inclusive i.e when time hits end time its over
    const PROBABILITY_WEIGHTING: f64 = 1.0;//0.4 with random
    //GA stuff
    const EVAL_ITER: u8 = 1;//Should always be 1 if using incident plan
    const SOL_NUM: u16 = 400;//Must be div by 4
    const TIMEOUT: u16 = 50;
    const MUTATION_NUM: u8 = 2;


    
    let total_steps: f32 = END_TIME/TIMESTEP;

    let mut spawn_stack: Vec<Vec<incident::Incident>> = Vec::new();
    let mut spawn_time: types::Time = 0.0;
    let mut incident_sum: usize = 0;
    for _ in 0..total_steps as usize{
        let mut step_incidents: Vec<Incident> = Vec::new();
        generate_incidents(&mut step_incidents, &incident_probs, TIMESTEP, spawn_time, PROBABILITY_WEIGHTING, &mut rng);
        incident_sum += step_incidents.len();
        spawn_stack.push(step_incidents);
        spawn_time += TIMESTEP;
    }
    println!("Incident plan generated containing {} incidents", incident_sum);

    for _ in 0..SOL_NUM{
        solutions.push(genetic::generate_solution(base_locations.len() as u8, MAX_CARS, &mut rng));
    }
    println!("Solutions generated");

    let mut unreachable_set: HashSet<types::Location> = HashSet::new();

    let start_time: Instant = Instant::now();
    let start: Data = avg_and_best_fitness(&solutions, 1, &mut spawn_stack, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME, &mut unreachable_set);
    println!("First evaluations in {} seconds", start_time.elapsed().as_secs_f32());

    for i in 0..TIMEOUT{
        evolve_pop(&mut solutions, SOL_NUM/2, SOL_NUM*3/4, EVAL_ITER, &spawn_stack, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME, MAX_CARS, MUTATION_NUM, &mut rng, &mut unreachable_set);
        println!("{}/{}", i + 1, TIMEOUT);
    }
    let end: Data = avg_and_best_fitness(&solutions, 1, &mut spawn_stack, &graph, &base_locations, &mut route_cache, TIMESTEP, END_TIME, &mut unreachable_set);

    let end_time: types::Time = start_time.elapsed().as_secs_f32();
    println!("*************\n{} iterations in {}", TIMEOUT + 1, end_time);
    println!("Averaging {} seconds per iteration\n          {} per 1000 solution evaluations", end_time/(TIMEOUT + 1) as f32, end_time as f64/(TIMEOUT + 1) as f64 / SOL_NUM as f64 * 1000.0);
    println!("*************\nStarting solutions:\n{}", start);

    println!("*************Final solutions:\n{}\n", end);

    let names: Vec<String> = read_data::police_names();
    let solution: &Solution = end.get_best_solution();
    for i in 0..solution.len(){
        println!("{}: {}", names[i], solution[i]);
    }

    println!("*************\nUnreachables: {:?}", unreachable_set);
}