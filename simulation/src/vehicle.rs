//! Module containg the [Vehicle] struct and it's associated methods.

use std::fmt;

use crate::types;

/// Represents a police vehicle that can navigate the graph and be ordered to respond to [Incident].
pub struct Vehicle{
    ///The [Node] ID of the vehicle's base.
    base: types::Location,
    ///The [Node] ID of the vehcile's current location.
    location: types::Location,
    ///The amounts of seconds until the vehicle is free for new commands.
    secs_till_free: types::Time,
    ///The name of the vehicle. Only ever used for logging or debugging.
    name: String,
    ///The cost of the last travel the vehicle performed.
    last_travel: types::Time,
    ///A boolean that is ture if the last travel a vehicle made was directly from it's base.
    just_come_from_base: bool,
}

impl Vehicle{
    /// Returns a new Vehicle object
    /// ### Parameters
    /// * `name` - The name of the vehicle.
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

    ///Returns the number of seconds until the vehicle is free to recieve new commands.
    pub fn get_secs_till_free(&self) -> types::Time{
        self.secs_till_free
    }
    ///Returns the node ID of the vehicle's current location.
    pub fn get_location(&self) -> types::Location{
        self.location
    }
    ///Returns the node ID of the vehicle's base
    pub fn get_base(&self) -> types::Location{
        self.base
    }
    ///Returns the name of the vehcile
    pub fn get_name(&self) -> String{
        self.name.clone()
    }

    /// Moves the vehicle to a new node.
    /// ### Parameters
    /// * `target` - The node ID of the location to move to
    /// * `travel_time` The time it takes for the vehcile to travel to the new location.
    /// * `incident_time` If the vehicle is going to an incident how long will they have to attend? If it's not an incident then set 0.
    /// ### Returns
    /// `Result<types::Location, &'static str>` - Either Ok or Err indicating if the move was successfull or not.
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
    /// Reduces the `secs_till_free` attribute by the indicated timestep.
    /// ### Parameters
    /// * `timestep` - Time value to reduces `secs_till_free` by.
    pub fn timestep(&mut self, timestep: types::Time){
        if timestep > self.secs_till_free{
            self.secs_till_free = 0.0;
        }
        else{
            self.secs_till_free = self.secs_till_free - timestep;
        }
    }
    /// Resets the vehcile back to it's base. The last_travel value is used as a guess to how long the vehcile should be occupied for.
    /// 
    /// This method should only be used as a last resort if a vehicle becomes trapped.
    pub fn reset(&mut self){
        self.location = self.base;
        self.secs_till_free = self.last_travel;
    }
    ///Returns a boolean value indicating if the vehicle's last move originated at it's base.
    pub fn last_move_from_base(&self) -> bool{
        self.just_come_from_base
    }
    ///Returns the travel_cost of the vehicle's last travel.
    pub fn get_last_travel(&self) -> types::Time{
        self.last_travel
    }
}

impl fmt::Display for Vehicle{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} reporting:\nLocation:{}\nTime till free:{}", self.name, self.location, self.secs_till_free)
    }
}