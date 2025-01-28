/// A ID number which corresponds to a OSM node's ID. There is no info relating to actual geographical positioning anywhere in this program.
pub type Location = u64;

/// A time value in seconds.
pub type Time = f32;

/// A solution is comprised of a number of ints.
/// 
/// Each index represents the number of police cars that should be stored at each base.
/// 
/// #### Example
/// [1,0,3,2]
/// 
/// This can be read as 1 car being stored at station 0, none at station 1, 3 at station 2, and 2 at station 3.
pub type Solution = Vec<u8>;