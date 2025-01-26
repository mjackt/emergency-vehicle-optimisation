use std::fmt;

use crate::types;

pub struct Data{
    best_solution: Vec<u8>,
    best_fitness: types::Time,
    avg_fitness: types::Time,
}

impl Data{
    pub fn new(best_solution: Vec<u8>, best_fitness: types::Time, avg_fitness: types::Time) -> Self{
        Self{
            best_solution,
            best_fitness,
            avg_fitness
        }
    }

    pub fn get_best_solution(&self) -> &Vec<u8>{
        &self.best_solution
    }
}

impl fmt::Display for Data{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Avg Fitness: {}\nBest Fitness: {}", self.avg_fitness, self.best_fitness)
    }
}