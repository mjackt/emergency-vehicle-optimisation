use crate::types;

#[derive(Clone)]
pub struct Incident{
    location: types::Location,
    service_time: types::Time,
    resolved_time: Option<types::Time>,
    creation_time: types::Time,
    valid : bool,
}

impl Incident{
    pub fn new(location: types::Location, service_time: types::Time, creation_time: types::Time) -> Self{
        Self{
            location,
            service_time,
            resolved_time : None,
            creation_time,
            valid: true,
        }
    }

    pub fn get_location(&self) -> types::Location{
        self.location
    }

    pub fn get_service_time(&self) -> types::Time{
        self.service_time
    }

    pub fn get_resolved_time(&self) -> Option<types::Time>{
        self.resolved_time
    }

    pub fn get_creation_time(&self) -> types::Time{
        self.creation_time
    }

    pub fn unreachable(&mut self){
        self.valid = false;
    }

    pub fn solved(&mut self, time: types::Time){
        self.resolved_time = Some(time);
    }

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

    pub fn is_valid(&self) -> bool{
        self.valid
    }
}