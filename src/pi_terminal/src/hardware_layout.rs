pub const DRIVETYPE_NONE: &str = "None";
pub const DRIVETYPE_CD: &str = "CD";
pub const DRIVETYPE_DVD: &str = "DVD";
pub const DRIVETYPE_BRAY: &str = "BRAY";
pub const DRIVETYPE_UHD: &str = "UHD";
pub const DRIVETYPE_HDDVD: &str = "HDDVD";

// these "locations" are ticks from 0 for the movement arm
pub const INPUT_SPINDLE_LOCATIONS: [i32; 4] = [0, 200, 400, 600];
pub const OUTPUT_SPINDLE_LOCATIONS: [i32; 4] = [1000, 1200, 1400, 1600];
pub const CAMERA_LOCATION: (i32, i32) = (500, 1500);
pub const SPINDLE_HEIGHT: i32 = 5000;
pub const CAMERA_PLATE_STEPS: i32 = 500;

/*
u8 has the max value of 255
u16 has the max value of 65535
u32 has the max value of 4294967295
u64 has the max value of 18446744073709551615

i32 has the max value of 2147483647
*/
  