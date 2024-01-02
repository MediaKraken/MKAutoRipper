pub const DRIVETYPE_NONE: &str = "None";
pub const DRIVETYPE_CD: &str = "CD";
pub const DRIVETYPE_DVD: &str = "DVD";
pub const DRIVETYPE_BRAY: &str = "BRAY";
pub const DRIVETYPE_UHD: &str = "UHD";
pub const DRIVETYPE_HDDVD: &str = "HDDVD";

// these "locations" are ticks from 0 for the movement arm
pub const INPUT_SPINDLE_LOCATIONS: [u64; 4] = [0, 200, 400, 600];
pub const OUTPUT_SPINDLE_LOCATIONS: [u64; 4] = [1000, 1200, 1400, 1600];
pub const CAMERA_LOCATION: (u64, u64) = (500, 1500);

/*
u8 has the max value of 255
u16 has the max value of 65535
u32 has the max value of 4294967295
u64 has the max value of 18446744073709551615
*/
  