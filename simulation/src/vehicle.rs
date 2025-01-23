use std::fmt;

use crate::types;

pub struct Vehicle{
    base: types::Location,
    location: types::Location,
    secs_till_free: types::Time,
    name: String,
    last_travel: types::Time,
    just_come_from_base: bool,
}

impl Vehicle{
    pub fn new(base: types::Location, name: String) -> Self{
        Self{
            base,
            location: base,
            secs_till_free: 0.0,
            name,
            last_travel: 0.0,
            just_come_from_base: false,
        }
    }

    pub fn get_secs_till_free(&self) -> types::Time{
        self.secs_till_free
    }

    pub fn get_location(&self) -> types::Location{
        self.location
    }

    pub fn get_base(&self) -> types::Location{
        self.base
    }

    pub fn get_name(&self) -> String{
        self.name.clone()
    }

    pub fn goto(&mut self, target: types::Location, travel_time: types::Time, incident_time: types::Time) -> Result<types::Location, &'static str> {
        if self.secs_till_free > 0.0{
            return Err("Busy")
        }

        else if target == self.location{
            return Err("Already Here")
        }
        //Check if its currently at base
        if self.location == self.base{
            self.just_come_from_base = true//Will stay true until next ordered move
        }
        else{
            self.just_come_from_base = false
        }
        // Update state
        self.location = target;
        self.secs_till_free = travel_time + incident_time;
        self.last_travel = travel_time;

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

    pub fn reset(&mut self){
        self.location = self.base;
        self.secs_till_free = self.last_travel;
    }

    pub fn last_move_from_base(&self) -> bool{
        self.just_come_from_base
    }

    pub fn get_last_travel(&self) -> types::Time{
        self.last_travel
    }
}

impl fmt::Display for Vehicle{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} reporting:\nLocation:{}\nTime till free:{}", self.name, self.location, self.secs_till_free)
    }
}