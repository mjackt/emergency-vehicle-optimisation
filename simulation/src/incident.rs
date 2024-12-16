use crate::types;

pub struct Incident{
    location: types::Location,
    service_time: types::Time,
    solved: bool,
}

impl Incident{
    pub fn new(location: types::Location, service_time: types::Time) -> Self{
        Self{
            location,
            service_time,
            solved: false
        }
    }

    pub fn get_location(&self) -> types::Location{
        self.location
    }

    pub fn get_service_time(&self) -> types::Time{
        self.service_time
    }

    pub fn solved(&mut self){
        self.solved = true;
    }

    pub fn is_solved(&self) -> bool{
        self.solved
    }
}