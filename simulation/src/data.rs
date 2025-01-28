//! Module containg the [Data] struct and it's associated methods.
use std::fmt;

use crate::types::{Solution, Time};

///Used to store the results from an evaluation of a collection of solutions.
pub struct Data{
    ///The solution that had the best overall fitness.
    best_solution: Solution,
    ///The fitness value of the best solution
    best_fitness: Time,
    ///The average fitness of all solutions
    avg_fitness: Time,
}

impl Data{
    ///Returns a new data object.
    pub fn new(best_solution: Solution, best_fitness: Time, avg_fitness: Time) -> Self{
        Self{
            best_solution,
            best_fitness,
            avg_fitness
        }
    }

    ///Returns a reference to the best solution
    pub fn get_best_solution(&self) -> &Solution{
        &self.best_solution
    }
}

impl fmt::Display for Data{
    ///Formats the struct by providing the average and best fitness values in string format.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Avg Fitness: {}\nBest Fitness: {}", self.avg_fitness, self.best_fitness)
    }
}