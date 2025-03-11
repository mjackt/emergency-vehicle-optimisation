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
//! Others refer to the simulation, i.e TIMESTEP or the max number of cars available.
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

use chrono::offset::Utc;
use chrono::DateTime;

use std::collections::HashMap;
use std::time::SystemTime;
use std::{env, io};
use std::fs::OpenOptions;
use data::Data;
use incident::Incident;
use node::Node;
use rand::rngs::ThreadRng;
use rand::rng;
use simulation::generate_incidents;
use weighted_rand::builder::*;
use weighted_rand::table::WalkerTable;
use std::io::Write;


/// Runs the entire GA process on input data pointed to by PLACE in read_data.rs.
/// 
/// After execution it will provide info on the GA process such as the final best solution, change in solution average and timing info.
fn main(){
    let args: Vec<String> = env::args().collect();

    if args.len() > 1{//Creates new csv file with station names as column headers
        println!("Enter 'y' to confirm a new results file");
        let mut txt = String::new();
     
        io::stdin().read_line(&mut txt).expect("failed to readline");
        if txt.trim() == "y"{
            println!("Creating new results.csv ...");
            let csv_row = read_data::police_names(PLACE).join(",");

            let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true) // Clears the file if it exists
            .open("results.csv").unwrap();

            // Write data as a new line
            writeln!(file, "{}", csv_row);
        }
        else {
            println!("Results will be appended to current file");
        }
    }
    else{
        println!("Results will be appended to current file");
    }

    let incident_probs: HashMap<types::Location, f32> = read_data::probs(PLACE);

    println!("{} nodes have more than 0 incidents per year", incident_probs.len());

    let graph: HashMap<types::Location, Node> = read_data::graph(PLACE);

    let base_locations: Vec<types::Location> = read_data::police(PLACE);

    let mut route_cache: HashMap<(types::Location, types::Location), types::Time> = HashMap::new();

    let mut rngthread: ThreadRng = rng();

    //TUNABLES
    const PARAM_TEST: bool = false;//Will run a selection of paramaters on the same schedule to compare results
    //Sim stuff
    const PLACE: &str = "dnc_12months_50agg_final";
    const MAX_CARS: u16 = 100;
    const TIMESTEP: types::Time = 60.0;
    const END_TIME: types::Time = 60.0 * 60.0 * 72.0;//Secs. Not inclusive. i.e when time hits end time its over
    const PROBABILITY_WEIGHTING: f64 = 1.0;//use 0.4 with random incidents
    const SEVERITY_WEIGHTING: [u32;5] = [50, 25, 12, 6, 0];//Weighting towards how many cars needed. Index 0 is weighting for 1 car, index 1 is weighting for 2 cars etc...
    const SERVICE_TIME_MEAN: f32 = 35.0;
    const SERVICE_TIME_STD: f32 = 9.0;//Currently not used
    //GA stuff
    const EVAL_ITER: u8 = 1;//Should always be 1 if using incident plan
    const SOL_NUM: u16 = 100;//Must be div by 4
    const TIMEOUT: u16 = 200;
    const MUTATION_NUM: u8 = 1;
    const MUTATION_NUM_WHEN_NO_XOVER: u8 = 0;
    const CROSSOVER_PROBABILITY_DECREASE: f32 = 0.0;
    const CROSSOVER_PROBABILITY: f32 = 1.0;
    const GA_RUNS: usize = 10;
    const TOURNAMENT_SIZE: u16 = 100;

    let builder: WalkerTableBuilder = WalkerTableBuilder::new(&SEVERITY_WEIGHTING);
    let mut wa_table: WalkerTable = builder.build();
    
    let total_steps: f32 = END_TIME/TIMESTEP;

    let mut results: Vec<Data> = Vec::new();
    if !PARAM_TEST{
        for i in 0..GA_RUNS{
            let system_time = SystemTime::now();
            let datetime: DateTime<Utc> = system_time.into();
            println!("{}", datetime.format("%d/%m/%Y %T"));
            let file_name: String = format!("fitness{}-{}.csv", i, datetime);
            genetic::run(&mut results, total_steps, &incident_probs, TIMESTEP, PROBABILITY_WEIGHTING, &mut rngthread, &mut wa_table, SERVICE_TIME_MEAN, SERVICE_TIME_STD, SOL_NUM, &base_locations, MAX_CARS, &graph, &mut route_cache, END_TIME, EVAL_ITER, PLACE, TIMEOUT, MUTATION_NUM, MUTATION_NUM_WHEN_NO_XOVER, CROSSOVER_PROBABILITY, CROSSOVER_PROBABILITY_DECREASE, None, TOURNAMENT_SIZE, &file_name);
        }
    }
    else{//Testing
        println!("GRID SEARCH");
        let mut spawn_stack: Vec<Vec<incident::Incident>> = Vec::new();
        let mut spawn_time: types::Time = 0.0;
        let mut incident_sum: usize = 0;
        let mut vehicle_count: [u32; 5] = [0; 5];
        for _ in 0..total_steps as usize{
            let mut step_incidents: Vec<Incident> = Vec::new();
            generate_incidents(&mut step_incidents, &incident_probs, TIMESTEP, spawn_time, PROBABILITY_WEIGHTING, &mut rngthread, &wa_table, &mut vehicle_count, SERVICE_TIME_MEAN, SERVICE_TIME_STD);
            incident_sum += step_incidents.len();
            spawn_stack.push(step_incidents);
            spawn_time += TIMESTEP;
        }
        println!("Uniform plan contains {} incidents", incident_sum);
        for i in 0..vehicle_count.len(){
            println!("{} incidents required {} cars", vehicle_count[i], i+1);
        }
        //                    E_I SO_N TIM  M_N M_N2 XO_P XO_D TSZ
        let grid_search: Vec<(u8, u16, u16, u8, u8, f32, f32, u16)> = vec![//Order of tunables match order in const declarations. For every tuple a GA run will be complted with those components
            (1,100,200,1,0,1.0, 0.0, 100),
            (1,100,200,1,2,1.0, 0.0025, 100),
            (1,100,200,1,2,1.0, 0.0030, 100),
            (1,100,200,1,2,1.0, 0.0035, 100),
        ];
        for i in 0..grid_search.len(){
            let file_name: String = format!("fitness{}.csv", i);
            genetic::run(&mut results, total_steps, &incident_probs, TIMESTEP, PROBABILITY_WEIGHTING, &mut rngthread, &mut wa_table, SERVICE_TIME_MEAN, SERVICE_TIME_STD, grid_search[i].1, &base_locations, MAX_CARS, &graph, &mut route_cache, END_TIME, grid_search[i].0, PLACE, grid_search[i].2, grid_search[i].3, grid_search[i].4, grid_search[i].5, grid_search[i].6, Some(spawn_stack.clone()), grid_search[i].7, &file_name);
        }
    }

    let mut sum_best_response_time: types::Time = 0.0;
    let mut sum_solutions: Vec<usize> = Vec::new();
    for _ in 0..base_locations.len(){
        sum_solutions.push(0);
    }
    for result in results{
        sum_best_response_time += result.get_best_fitness();
        let best_solution = result.get_best_solution();
        for i in 0..best_solution.len(){
            sum_solutions[i] += best_solution[i] as usize;
        }
    }

    println!("*************\nAverages:\nBest Fitness: {}", sum_best_response_time/GA_RUNS as f32);

    let names: Vec<String> = read_data::police_names(PLACE);
    let mut outcomes: Vec<(String, f32)> = Vec::new();
    for i in 0..sum_solutions.len(){
        outcomes.push((names[i].clone(), sum_solutions[i] as f32/GA_RUNS as f32));
    }

    outcomes.sort_by(|a, b| a.0.cmp(&b.0));

    for element in outcomes{
        println!("{}: {}", element.0, element.1);
    }
}