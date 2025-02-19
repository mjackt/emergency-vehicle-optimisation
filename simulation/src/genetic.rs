//! Module containing all the functions related to the genetic algorithm operation.
use std::{collections::{HashMap, HashSet}, f32::MAX, fs::OpenOptions, time::Instant};
use rand::{distr::{Distribution, Uniform}, rng, rngs::ThreadRng, seq::SliceRandom, Rng};
use weighted_rand::table::WalkerTable;
use crate::{data::Data, genetic, incident::{self, Incident}, node::Node, read_data, simulation::{evaluate, generate_incidents}, types::{self, Solution}};

use std::io::Write;

/// Returns a solution by randomly assigning a number of cars to a number of police bases.
/// ### Parameters
/// * `base_number` - Number of police stations or bases in use.
/// * `max_cars` - Maximum cars available i.e number of cars to be assigned to bases.
/// * `rngthread` - A ThreadRng object to avoid constant reseeding.
/// ### Returns
/// A `Vec<u8>` of length *n* whose values will sum to *x*, where *n* is `base_number` and *x* is `max_cars`.
pub fn generate_solution(base_number: u8, max_cars: u16, rngthread: &mut ThreadRng) -> Solution{
    let mut solution: Solution = vec![0; base_number as usize];

    let selection: Uniform<u8> = Uniform::try_from(0..base_number).unwrap();

    for _ in 0..max_cars{
        let choice: u8 = selection.sample(rngthread);
        solution[choice as usize] += 1;
    }

    return solution
}

/// Returns a list of indexes of the selected solutions after running a tournement selection with specified parameters.
/// ### Parameters
/// * `solutions` - A Vec of solutions to run tournament selection on.
/// * `num_to_select` - The number of solutions to be selected.
/// * `tournament_size` - The number of solutions to be entered into the tournament.
/// * `eval_iterations` - The number of iterations to evaluate each solution for.
/// * `spawn_stack` - The incident schedule.
/// * `graph` - The graph of nodes.
/// * `base_locations` - Vec of locations ids containg every police base.
/// * `route_cache` - The cache of route calculations.
/// * `timestep` - Timestep to run the simulation with.
/// * `end_time` - Time to end simulation at.
/// * `rngthread` - A ThreadRng object to avoid constant reseeding.
/// * `unreachable_set` - HashSet containg ids of every node that is unreachable.
/// * `best_evaluated_fitness` - A reference to a Time value that will be updated with the best fitness discovered from the round of evaluations
/// ### Returns
/// A list of indexes of the solutions to be selected. Corresponds to the `solutions` parameter.
pub fn tournament_selection(solutions: &Vec<Solution>,
                            num_to_select: u16,
                            tournament_size: u16,
                            eval_iterations: u8,
                            spawn_stack: &Vec<Vec<incident::Incident>>,
                            graph: &HashMap<types::Location, Node>,
                            base_locations: &Vec<types::Location>,
                            route_cache: &mut HashMap<(types::Location, types::Location), types::Time>,
                            timestep: types::Time,
                            end_time: types::Time,
                            rngthread: &mut ThreadRng,
                            unreachable_set: &mut HashSet<types::Location>,
                            best_evaluated_fitness: &mut types::Time,
                            ) -> Vec<usize>{

    let mut selected: Vec<usize> = Vec::new();
    let mut selected_fitness: Vec<types::Time> = Vec::new();

    let uniform_rng: Uniform<usize> = Uniform::try_from(0..solutions.len()).unwrap();
    *best_evaluated_fitness = MAX;

    for _ in 0..tournament_size{
        let mut choice: usize = uniform_rng.sample(rngthread);

        while selected.contains(&choice){
            choice = uniform_rng.sample(rngthread);
        }

        let tournament_entry: &Solution = &solutions[choice];
        let solution_fitness: types::Time = evaluate(tournament_entry, eval_iterations, spawn_stack, graph, base_locations, route_cache, timestep, end_time, unreachable_set);
        
        if solution_fitness < *best_evaluated_fitness{
            *best_evaluated_fitness = solution_fitness;
        }

        for i in 0..num_to_select as usize{
            match selected_fitness.get(i){
                Some(fitness) => {
                    if solution_fitness < *fitness{
                        selected.insert(i, choice);
                        selected_fitness.insert(i, solution_fitness);

                        if selected.len() > num_to_select as usize{
                            selected.pop();
                            selected_fitness.pop();
                        }
                        break;
                    }
                }
                None => {//selection not filled yet
                    selected.push(choice);
                    selected_fitness.push(solution_fitness);
                    break;
                }
            }
        }
    }
    selected
}

/// Mutates a given solution by "taking" a count from one base and giving it to another. Theerfore it doesn't require repair.
/// ### Parameters
/// * `solution` - Solution to be mutated.
/// * `num_of_mutations` - The number of indexes to be mutated.
/// * `rngthread` - A ThreadRng object to avoid constant reseeding.
fn mutate(solution: &mut Solution, num_of_mutations: u8, rngthread: &mut ThreadRng){
    let uniform_rng: Uniform<usize> = Uniform::try_from(0..solution.len()).unwrap();
    for _ in 0..num_of_mutations{
        let mut index = uniform_rng.sample(rngthread);
        if solution[index] > 0 {
            solution[index] -= 1;
            index = uniform_rng.sample(rngthread);
            solution[index] += 1;
        }
    }
}

/// Uniform crossover method for two solutions `a` and `b`. A random repair system is used to ensure children solution adhere to `max_cars` constraint.
/// * `a` - A solution to be crossed.
/// * `b` - The other solution to be crossed.
/// * `max_cars` - Maximum cars available.
/// * `rngthread` - A ThreadRng object to avoid constant reseeding.
pub fn crossover(a: &mut Solution, b: &mut Solution, max_cars: u16, rngthread: &mut ThreadRng){
    for i in 0..a.len(){
        let random_bool: bool = rngthread.random::<bool>();
        if random_bool == true{
            let a_value: u8 = a[i];

            a[i] = b[i];
            b[i] = a_value;
        }
    }

    repair(a, max_cars, rngthread);
    repair(b, max_cars, rngthread);
}

/// Randomly repairs a solution (if required), to ensure adherence with `max_cars` constraint
/// * `solution` - Solution to be repaired.
/// * `max_cars` - Maximum cars available.
/// * `rngthread` - A ThreadRng object to avoid constant reseeding.
fn repair(solution: &mut Solution, max_cars: u16, rngthread: &mut ThreadRng){
    let mut total_sum: i16 = 0;
    for i in 0..solution.len(){
        total_sum += solution[i] as i16;
    }

    let change: i8;

    if total_sum == max_cars as i16{
        return
    }
    else if total_sum > max_cars as i16{
        change = -1;
    }
    else{
        change = 1;
    }
    let uniform_rng: Uniform<usize> = Uniform::try_from(0..solution.len()).unwrap();

    while total_sum != max_cars as i16{
        let index = uniform_rng.sample(rngthread);
        if solution[index] > 0{
            let temp: i8 = solution[index].clone() as i8 + change;
            solution[index] = temp as u8;
            total_sum += change as i16;
        }
    }
}

/// Evolves a population of solutions using:
/// - Tournament selection
/// - Crossover (and repair)
/// - Mutation
/// ### Parameters
/// * `solutions` - A Vec of solutions to evolve
/// * `num_to_select` - The number of solutions to be selected from tournament selection.
/// * `tournament_size` - The number of solutions to be entered into the tournament.
/// * `eval_iterations` - The number of iterations to evaluate each solution for.
/// * `spawn_stack` - The incident schedule.
/// * `graph` - The graph of nodes.
/// * `base_locations` - Vec of locations ids containg every police base.
/// * `route_cache` - The cache of route calculations.
/// * `timestep` - Timestep to run the simulation with.
/// * `end_time` - Time to end simulation at.
/// * `max_cars` - Maximum cars available.
/// * `num_of_mutations` - The number of indexes to be mutated.
/// * `num_of_mutations_when_no_xover` - The number of indexes to be mutated if do_crossover is *False*.
/// * `rngthread` - A ThreadRng object to avoid constant reseeding.
/// * `unreachable_set` - HashSet containg ids of every node that is unreachable.
/// * `do_crossover` - Use crossover to evolve the population or not.
pub fn evolve_pop(solutions: &mut Vec<Solution>,
                    num_to_select: u16,
                    tournament_size: u16,
                    eval_iterations: u8,
                    spawn_stack: &Vec<Vec<incident::Incident>>,
                    graph: &HashMap<types::Location, Node>,
                    base_locations: &Vec<types::Location>,
                    route_cache: &mut HashMap<(types::Location, types::Location), types::Time>,
                    timestep: types::Time,
                    end_time: types::Time,
                    max_cars: u16,
                    num_of_mutations: u8,
                    num_of_mutations_when_no_xover: u8,
                    rngthread: &mut ThreadRng,
                    unreachable_set: &mut HashSet<types::Location>,
                    do_crossover: bool,
                ) -> types::Time{
    let mut best_evaluated_fitness: types::Time = std::f32::MAX;
    let mut selected_indexes: Vec<usize> = tournament_selection(solutions, num_to_select, tournament_size, eval_iterations, spawn_stack, graph, base_locations, route_cache, timestep, end_time, rngthread, unreachable_set, &mut best_evaluated_fitness);
    selected_indexes.shuffle(&mut rng());

    let mut new_solutions: Vec<Solution> = Vec::new();

    for i in 0..selected_indexes.len(){
        let selected_index: usize = selected_indexes[i];
        new_solutions.push(solutions[selected_index].clone());

        if i % 2 == 0{
            let next_selected_index: usize = selected_indexes[i+1];
            let mut a = solutions[selected_index].clone();
            let mut b = solutions[next_selected_index].clone();
            
            if do_crossover{
                crossover(&mut a, &mut b, max_cars, rngthread);
                mutate(&mut a, num_of_mutations, rngthread);
                mutate(&mut b, num_of_mutations, rngthread);
            }
            else{
                mutate(&mut a, num_of_mutations_when_no_xover, rngthread);
                mutate(&mut b, num_of_mutations_when_no_xover, rngthread);
            }

            new_solutions.push(a);
            new_solutions.push(b);
        }
    }

    *solutions = new_solutions;
    best_evaluated_fitness
}

/// Evaluates a population of solutions and returns a [Data] struct containing info on best and average solution performance.
/// ### Parameters
/// * `solutions` - A Vec of solutions to evolve
/// * `eval_iterations` - The number of iterations to evaluate each solution for.
/// * `spawn_stack` - The incident schedule.
/// * `graph` - The graph of nodes.
/// * `base_locations` - Vec of locations ids containg every police base.
/// * `route_cache` - The cache of route calculations.
/// * `timestep` - Timestep to run the simulation with.
/// * `end_time` - Time to end simulation at.
/// * `unreachable_set` - HashSet containg ids of every node that is unreachable.
/// ### Returns
/// A [Data] struct containing info on best and average solution performance.
pub fn avg_and_best_fitness(solutions: &Vec<Solution>,
                            eval_iterations: u8,
                            spawn_stack: &Vec<Vec<incident::Incident>>,
                            graph: &HashMap<types::Location, Node>,
                            base_locations: &Vec<types::Location>,
                            route_cache: &mut HashMap<(types::Location, types::Location), types::Time>,
                            timestep: types::Time,
                            end_time: types::Time,
                            unreachable_set: &mut HashSet<types::Location>) 
                            -> Data{
    let mut best_solution: &Solution = &solutions[0];
    let mut best_fitness: types::Time = evaluate(&solutions[0], eval_iterations, spawn_stack, graph, base_locations, route_cache, timestep, end_time, unreachable_set);

    let mut total_fitness: types::Time = 0.0;


    for i in 1..solutions.len(){
        let fit: types::Time = evaluate(&solutions[i], eval_iterations, spawn_stack, graph, base_locations, route_cache, timestep, end_time, unreachable_set);
        total_fitness += fit;
        if fit < best_fitness{
            best_fitness = fit;
            best_solution = &solutions[i];
        }
    }

    Data::new(best_solution.clone(), best_fitness, total_fitness/solutions.len() as f32)
}


pub fn run(results: &mut Vec<Data>,
            total_steps: f32,
            incident_probs: &HashMap<types::Location, types::Time>,
            timestep: types::Time,
            probability_weighting: f64,
            rngthread: &mut ThreadRng,
            wa_table: &mut WalkerTable,
            service_time_mean: types::Time,
            service_time_std: types::Time,
            sol_num: u16,
            base_locations: &Vec<types::Location>,
            max_cars: u16,
            graph: &HashMap<types::Location, Node>,
            route_cache: &mut HashMap<(types::Location, types::Location), types::Time>,
            end_time: types::Time,
            eval_iter: u8,
            place: &str,
            timeout: u16,
            mutation_num: u8,
            mutation_num_when_no_xover: u8,
            crossover_prob: f32,
            spawn_stack_option: Option<Vec<Vec<incident::Incident>>>,
            tournament_size: u16,
            ){
    
    let mut solutions : Vec<Solution> = Vec::new();
    let mut spawn_stack: Vec<Vec<incident::Incident>> = Vec::new();
    let mut spawn_time: types::Time = 0.0;
    let mut incident_sum: usize = 0;
    let mut vehicle_count: [u32; 5] = [0; 5];
    match spawn_stack_option{
        None => {
            for _ in 0..total_steps as usize{
                let mut step_incidents: Vec<Incident> = Vec::new();
                generate_incidents(&mut step_incidents, &incident_probs, timestep, spawn_time, probability_weighting, rngthread, &wa_table, &mut vehicle_count, service_time_mean, service_time_std);
                incident_sum += step_incidents.len();
                spawn_stack.push(step_incidents);
                spawn_time += timestep;
            }
            println!("Incident plan generated containing {} incidents", incident_sum);
            for i in 0..vehicle_count.len(){
                println!("{} incidents required {} cars", vehicle_count[i], i+1);
            }
        }
        //Used to test different parameters on same spawn schedule
        Some(contained_spawn_stack) =>{
            spawn_stack = contained_spawn_stack;
        }
    }

    for _ in 0..sol_num{
        solutions.push(genetic::generate_solution(base_locations.len() as u8, max_cars, rngthread));
    }
    println!("*************\nSolutions generated");

    let mut unreachable_set: HashSet<types::Location> = HashSet::new();

    let start_time: Instant = Instant::now();
    let start: Data = avg_and_best_fitness(&solutions, 1, &mut spawn_stack, &graph, &base_locations, route_cache, timestep, end_time, &mut unreachable_set);
    println!("First evaluations in {} seconds", start_time.elapsed().as_secs_f32());

    for i in 0..timeout{
        let mut xover: bool = false;
        if rngthread.random::<f32>() <= crossover_prob{
            xover = true;
        }
        let evol_best_fitness:types::Time = evolve_pop(&mut solutions, sol_num/2, tournament_size, eval_iter, &spawn_stack, &graph, &base_locations, route_cache, timestep, end_time, max_cars, mutation_num, mutation_num_when_no_xover, rngthread, &mut unreachable_set, xover);
        println!("{}/{} ---{}--- {}", i + 1, timeout, xover, evol_best_fitness);
    }
    let end: Data = avg_and_best_fitness(&solutions, 1, &mut spawn_stack, &graph, &base_locations, route_cache, timestep, end_time, &mut unreachable_set);

    let end_time: types::Time = start_time.elapsed().as_secs_f32();
    println!("*************\n{} iterations in {}", timeout + 1, end_time);
    println!("Averaging {} seconds per iteration\n          {} per 1000 solution evaluations", end_time/(timeout + 1) as f32, end_time as f64/(timeout + 1) as f64 / sol_num as f64 * 1000.0);
    println!("*************\nStarting solutions:\n{}", start);

    println!("*************\nFinal solutions:\n{}\n", end);

    let names: Vec<String> = read_data::police_names(place);
    let solution: &Solution = end.get_best_solution();

    let csv_row = solution.iter()
    .map(|byte| byte.to_string()) // Convert each byte to a string
    .collect::<Vec<String>>() // Collect into Vec<String>
    .join(","); // Join with commas

    let mut file = OpenOptions::new()
    .create(true)
    .append(true)
    .open("results.csv").unwrap();

    // Write data as a new line
    writeln!(file, "{}", csv_row);

    let mut outcomes: Vec<(String, u8)> = Vec::new();
    for i in 0..solution.len(){
        outcomes.push((names[i].clone(), solution[i]));
    }

    outcomes.sort_by(|a, b| a.0.cmp(&b.0));

    for element in outcomes{
        println!("{}: {}", element.0, element.1);
    }

    println!("*************\nUnreachables: {:?}", unreachable_set);
    results.push(end);
}