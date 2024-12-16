use std::fmt;

use crate::types;

pub struct Vehicle{
    location: types::Location,
    secs_till_free: types::Time,
    name: String,
}

impl Vehicle{
    pub fn new(location: types::Location) -> Self{
        Self{
            location,
            secs_till_free: 0.0,
            name: "Car".to_string(),
        }
    }

    pub fn get_secs_till_free(&self) -> types::Time{
        self.secs_till_free
    }

    pub fn get_location(&self) -> types::Location{
        self.location
    }

    pub fn goto(&mut self, target: types::Location, travel_time: types::Time, incident_time: types::Time) -> Result<types::Location, &'static str> {
        if self.secs_till_free > 0.0{
            return Err("Busy")
        }

        else if target == self.location{
            return Err("Already Here")
        }

        // Update state
        self.location = target;
        self.secs_till_free = travel_time + incident_time;

        Ok(target)
    }

    pub fn timestep(&mut self, timestep: types::Time){
        if timestep > self.secs_till_free{
            self.secs_till_free = 0.0;
        }
        else{
            self.secs_till_free = self.secs_till_free - timestep;
        }
    }
}

impl fmt::Display for Vehicle{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} reporting:\nLocation:{}\nTime till free:{}", self.name, self.location, self.secs_till_free)
    }
}