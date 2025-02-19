//! Module containg the [Incident] struct and it's associated methods.

use crate::types;

///Represents an incident/crime that police vehicles can respond to in the simulation.
#[derive(Clone)]
pub struct Incident{
    ///The [Node] ID where the incident is taking place.
    location: types::Location,
    ///How long will the police need to attend the incident for until it's resolved.
    service_time: types::Time,
    ///The number of police vehicles required to resolve the incident.
    vehicles_required: usize,
    ///The time between the incident's creation time and the police's arrival. Will be `None` when not resolved and `Some<Time>` after police attendance.
    resolved_time: Option<types::Time>,
    ///The time the incident was created.
    creation_time: types::Time,
    ///Is the incident reachable. Some incidents may spawn in disconnected areas of the graph with no police cars.
    valid : bool,
}

impl Incident{
    ///Returns an Incident object.
    pub fn new(location: types::Location, service_time: types::Time, creation_time: types::Time, vehicles_required: usize) -> Self{
        Self{
            location,
            service_time,
            resolved_time : None,
            creation_time,
            valid: true,
            vehicles_required,
        }
    }

    ///Returns the incident's [Node] ID.
    pub fn get_location(&self) -> types::Location{
        self.location
    }

    ///Returns the service time of the incident
    pub fn get_service_time(&self) -> types::Time{
        self.service_time
    }

    pub fn get_vehicles_required(&self) -> usize{
        self.vehicles_required
    }

    ///Returns the time the incident was resolved in `Some<Time>` or `None` if the incident is yet to be resolved.
    pub fn get_resolved_time(&self) -> Option<types::Time>{
        self.resolved_time
    }

    ///Returns the creation time of the incident.
    pub fn get_creation_time(&self) -> types::Time{
        self.creation_time
    }

    ///Sets the incident as unreachable (invalid).
    pub fn unreachable(&mut self){
        self.valid = false;
    }

    /// Sets the incident as resolved.
    /// ### Parameters
    /// * `time` - The time the incident was resolved.
    pub fn solved(&mut self, time: types::Time){
        self.resolved_time = Some(time);
    }

    /// Returns a boolean specifying if the incident is already solved or not.
    pub fn is_solved(&self) -> bool{
        if self.valid{
            match self.resolved_time{
                Some(_) => true,
                None => false,
            }
        }
        else{
            true
        }
    }

    ///Returns a boolean specifying if the incident is reachable (valid) or not.
    pub fn is_valid(&self) -> bool{
        self.valid
    }
}