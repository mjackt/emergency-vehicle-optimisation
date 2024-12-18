use crate::types;

pub struct Incident{
    location: types::Location,
    service_time: types::Time,
    resolved_time: Option<types::Time>,
    creation_time: types::Time,
}

impl Incident{
    pub fn new(location: types::Location, service_time: types::Time, creation_time: types::Time) -> Self{
        Self{
            location,
            service_time,
            resolved_time : None,
            creation_time,
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

    pub fn solved(&mut self, time: types::Time){
        self.resolved_time = Some(time);
    }

    pub fn is_solved(&self) -> bool{
        match self.resolved_time{
            Some(_) => true,
            None => false,
        }
    }
}