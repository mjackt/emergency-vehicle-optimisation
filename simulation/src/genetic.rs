//! Module containing all the functions related to the genetic algorithm operation.
use std::collections::{HashMap, HashSet};
use rand::{distributions::{Uniform, Distribution}, rngs::ThreadRng, Rng, seq::SliceRandom, thread_rng};

use crate::{data::Data, incident, node::Node, simulation::evaluate, types::{self, Solution}};

/// Returns a solution by randomly assigning a number of cars to a number of police bases.
/// ### Parameters
/// * `base_number` - Number of police stations or bases in use.
/// * `max_cars` - Maximum cars available i.e number of cars to be assigned to bases.
/// * `rng` - A ThreadRng object to avoid constant reseeding.
/// ### Returns
/// A `Vec<u8>` of length *n* whose values will sum to *x*, where *n* is `base_number` and *x* is `max_cars`.
pub fn generate_solution(base_number: u8, max_cars: u16, rng: &mut ThreadRng) -> Solution{
    let mut solution: Solution = vec![0; base_number as usize];

    let selection: Uniform<u8> = Uniform::from(0..base_number);

    for _ in 0..max_cars{
        let choice: u8 = selection.sample(rng);
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
/// * `rng` - A ThreadRng object to avoid constant reseeding.
/// * `unreachable_set` - HashSet containg ids of every node that is unreachable.
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
                            rng: &mut ThreadRng,
                            unreachable_set: &mut HashSet<types::Location>,
                            ) -> Vec<usize>{

    let mut selected: Vec<usize> = Vec::new();
    let mut selected_fitness: Vec<types::Time> = Vec::new();

    let uniform_rng: Uniform<usize> = Uniform::from(0..solutions.len());

    for _ in 0..tournament_size{
        let mut choice: usize = uniform_rng.sample(rng);

        while selected.contains(&choice){
            choice = uniform_rng.sample(rng);
        }

        let tournament_entry: &Solution = &solutions[choice];
        let solution_fitness = evaluate(tournament_entry, eval_iterations, spawn_stack, graph, base_locations, route_cache, timestep, end_time, unreachable_set);

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
/// * `rng` - A ThreadRng object to avoid constant reseeding.
fn mutate(solution: &mut Solution, num_of_mutations: u8, rng: &mut ThreadRng){
    let uniform_rng: Uniform<usize> = Uniform::from(0..solution.len());
    for _ in 0..num_of_mutations{
        let mut index = uniform_rng.sample(rng);
        if solution[index] > 0 {
            solution[index] -= 1;
            index = uniform_rng.sample(rng);
            solution[index] += 1;
        }
    }
}

/// Uniform crossover method for two solutions `a` and `b`. A random repair system is used to ensure children solution adhere to `max_cars` constraint.
/// * `a` - A solution to be crossed.
/// * `b` - The other solution to be crossed.
/// * `max_cars` - Maximum cars available.
/// * `rng` - A ThreadRng object to avoid constant reseeding.
pub fn crossover(a: &mut Solution, b: &mut Solution, max_cars: u16, rng: &mut ThreadRng){
    for i in 0..a.len(){
        let random_bool: bool = rng.gen::<bool>();
        if random_bool == true{
            let a_value: u8 = a[i];

            a[i] = b[i];
            b[i] = a_value;
        }
    }

    repair(a, max_cars, rng);
    repair(b, max_cars, rng);
}

/// Randomly repairs a solution (if required), to ensure adherence with `max_cars` constraint
/// * `solution` - Solution to be repaired.
/// * `max_cars` - Maximum cars available.
/// * `rng` - A ThreadRng object to avoid constant reseeding.
fn repair(solution: &mut Solution, max_cars: u16, rng: &mut ThreadRng){
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
    let uniform_rng: Uniform<usize> = Uniform::from(0..solution.len());

    while total_sum != max_cars as i16{
        let index = uniform_rng.sample(rng);
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
/// * `rng` - A ThreadRng object to avoid constant reseeding.
/// * `unreachable_set` - HashSet containg ids of every node that is unreachable.
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
                    rng: &mut ThreadRng,
                    unreachable_set: &mut HashSet<types::Location>,
                ){

    let mut selected_indexes: Vec<usize> = tournament_selection(solutions, num_to_select, tournament_size, eval_iterations, spawn_stack, graph, base_locations, route_cache, timestep, end_time, rng, unreachable_set);
    selected_indexes.shuffle(&mut thread_rng());

    let mut new_solutions: Vec<Solution> = Vec::new();

    for i in 0..selected_indexes.len(){
        let selected_index: usize = selected_indexes[i];
        new_solutions.push(solutions[selected_index].clone());

        if i % 2 == 0{
            let next_selected_index: usize = selected_indexes[i+1];
            let mut a = solutions[selected_index].clone();
            let mut b = solutions[next_selected_index].clone();

            crossover(&mut a, &mut b, max_cars, rng);
            mutate(&mut a, num_of_mutations, rng);
            mutate(&mut b, num_of_mutations, rng);

            new_solutions.push(a);
            new_solutions.push(b);
        }
    }

    *solutions = new_solutions;
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