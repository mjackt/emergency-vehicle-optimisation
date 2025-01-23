use std::collections::HashMap;

use rand::{distributions::{Uniform, Distribution}, rngs::ThreadRng, Rng, seq::SliceRandom, thread_rng};

use crate::{incident, node::Node, simulation::evaluate, types};

pub fn generate_solution(base_number: u8, max_cars: u16, rng: &mut ThreadRng) -> Vec<u8>{
    let mut solution: Vec<u8> = vec![0; base_number as usize];

    let selection: Uniform<u8> = Uniform::from(0..base_number);

    for i in 0..max_cars{
        let choice: u8 = selection.sample(rng);
        solution[choice as usize] += 1;
    }

    return solution
}

pub fn tournament_selection(solutions: &Vec<Vec<u8>>, num_to_select: u16, tournament_size: u16,
                            eval_iterations: u8,
                            spawn_stack: &Vec<Vec<incident::Incident>>,
                            graph: &HashMap<types::Location, Node>,
                            base_locations: &Vec<types::Location>,
                            route_cache: &mut HashMap<(types::Location, types::Location), types::Time>,
                            timestep: types::Time,
                            end_time: types::Time,
                            rng: &mut ThreadRng,
                            ) -> Vec<usize>{//Vec of indexes to be selected

    let mut selected: Vec<usize> = Vec::new();
    let mut selected_fitness: Vec<types::Time> = Vec::new();

    let uniform_rng: Uniform<usize> = Uniform::from(0..solutions.len());

    for j in 0..tournament_size{
        let mut choice: usize = uniform_rng.sample(rng);

        while selected.contains(&choice){
            choice = uniform_rng.sample(rng);
        }

        let tournament_entry: &Vec<u8> = &solutions[choice];
        let solution_fitness = evaluate(tournament_entry, eval_iterations, spawn_stack, graph, base_locations, route_cache, timestep, end_time);

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

fn mutate(solution: &mut Vec<u8>, num_of_mutations: u8, rng: &mut ThreadRng){
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

//uniform crossover with random repair
pub fn crossover(a: &mut Vec<u8>, b: &mut Vec<u8>, max_cars: u16, rng: &mut ThreadRng){
    for i in 0..a.len(){
        let random_bool = rng.gen::<bool>();
        if random_bool == true{
            let a_value: u8 = a[i];

            a[i] = b[i];
            b[i] = a_value;
        }
    }

    repair(a, max_cars, rng);
    repair(b, max_cars, rng);
}

fn repair(solution: &mut Vec<u8>, max_cars: u16, rng: &mut ThreadRng){
    let mut total_sum: i8 = 0;
    for i in 0..solution.len(){
        total_sum += solution[i] as i8;
    }

    let mut change: i8 = 1;

    if total_sum == max_cars as i8{
        return
    }
    else if total_sum > max_cars as i8{
        change = -1;
    }
    else{
        change = 1;
    }
    let uniform_rng: Uniform<usize> = Uniform::from(0..solution.len());

    while total_sum != max_cars as i8{
        let index = uniform_rng.sample(rng);
        if solution[index] > 0{
            let temp: i8 = solution[index].clone() as i8 + change;
            solution[index] = temp as u8;
            total_sum += change;
        }
    }
}

pub fn evolve_pop(solutions: &mut Vec<Vec<u8>>,
                    num_to_select: u16,
                    tournament_size: u16,
                    eval_iterations: u8,
                    spawn_stack: &Vec<Vec<incident::Incident>>,
                    graph: &HashMap<types::Location, Node>, base_locations: &Vec<types::Location>,
                    route_cache: &mut HashMap<(types::Location, types::Location), types::Time>,
                    timestep: types::Time,
                    end_time: types::Time,
                    max_cars: u16,
                    num_of_mutations: u8,
                    rng: &mut ThreadRng,){

    let mut selected_indexes: Vec<usize> = tournament_selection(solutions, num_to_select, tournament_size, eval_iterations, spawn_stack, graph, base_locations, route_cache, timestep, end_time, rng);
    selected_indexes.shuffle(&mut thread_rng());

    let mut new_solutions: Vec<Vec<u8>> = Vec::new();

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

pub fn avg_and_best_fitness(solutions: &Vec<Vec<u8>>, eval_iterations: u8, spawn_stack: &Vec<Vec<incident::Incident>>, graph: &HashMap<types::Location, Node>, base_locations: &Vec<types::Location>, route_cache: &mut HashMap<(types::Location, types::Location), types::Time>, timestep: types::Time, end_time: types::Time){
    let mut best_soltuion = &solutions[0];
    let mut best_fitness = evaluate(&solutions[0], eval_iterations, spawn_stack, graph, base_locations, route_cache, timestep, end_time);

    let mut total_fitness: types::Time = 0.0;


    for i in 1..solutions.len(){
        let fit = evaluate(&solutions[i], eval_iterations, spawn_stack, graph, base_locations, route_cache, timestep, end_time);
        total_fitness += fit;
        if fit < best_fitness{
            best_fitness = fit;
            best_soltuion = &solutions[i];
        }
    }

    println!("Best fitness: {}, from: {:?}", best_fitness, best_soltuion);
    println!("Avg fitness: {}", total_fitness/solutions.len() as f32);
}