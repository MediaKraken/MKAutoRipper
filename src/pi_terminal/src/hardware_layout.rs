#[non_exhaustive]
pub struct DriveType;

impl DriveType {
    pub const CD: i16 = 1;
    pub const DVD: i16 = 2;
    pub const BRAY: i16 = 3;
    pub const UHD: i16 = 4;
    pub const HDDVD: i16 = 5;
}

lazy_static! {
    // these "locations" are ticks from 0 for the movement arm
    let input_spindle_locations: [i32; 4] = [0, 200, 400, 600];
    let output_spindle_locations: [i32; 4] = [1000, 1200, 1400, 1600];
}
