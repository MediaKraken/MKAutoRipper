pub const DRIVETYPE_NONE: &str = "None";
pub const DRIVETYPE_CD: &str = "CD";
pub const DRIVETYPE_DVD: &str = "DVD";
pub const DRIVETYPE_BRAY: &str = "BRAY";
pub const DRIVETYPE_UHD: &str = "UHD";
pub const DRIVETYPE_HDDVD: &str = "HDDVD";

// these "locations" are ticks from 0 for the movement arm
pub const INPUT_SPINDLE_LOCATIONS: [i32; 3] = [0, 160000, 315000];
pub const OUTPUT_SPINDLE_LOCATIONS: [i32; 4] = [1283000, 1440000, 1600000, 1756000];
pub const CAMERA_LOCATION: (i32, i32) = (0, 0);
pub const SPINDLE_HEIGHT: i32 = 0;
pub const CAMERA_PLATE_STEPS: i32 = 0;

pub const DRIVE_COLUMN_LOCATIONS: [i32; 5] = [516500, 709500, 892500, 1083500, 1066000];
pub const DRIVE_ROW_LOCATIONS: [i32; 7] = [0, 0, 0, 0, 0, 0, 0];

// BCM pin numbering! Do not use physcial pin numbers.
// Main movement arm
pub const GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT: u8 = 13; // 33
pub const GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT: u8 = 25; // 22
pub const GPIO_STEPPER_HORIZONTAL_DIRECTION: u8 = 26; // 37
pub const GPIO_STEPPER_HORIZONTAL_PULSE: u8 = 19; // 35
pub const GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED: u64 = 75;

// CD Picker/loader
pub const GPIO_STEPPER_VERTICAL_END_STOP_ASSEMBLY: u8 = 27; // 13
pub const GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM: u8 = 22; // 15
pub const GPIO_STEPPER_VERTICAL_END_STOP_TOP: u8 = 10; // 19
pub const GPIO_STEPPER_VERTICAL_DIRECTION: u8 = 11; // 23
pub const GPIO_STEPPER_VERTICAL_PULSE: u8 = 9; // 21
pub const GPIO_STEPPER_VERTICAL_MOTOR_SPEED: u64 = 50;

// Image tray
pub const GPIO_STEPPER_TRAY_END_STOP_BACK: u8 = 17; // 11
pub const GPIO_STEPPER_TRAY_END_STOP_FRONT: u8 = 18; // 12
pub const GPIO_STEPPER_TRAY_DIRECTION: u8 = 24; // 18
pub const GPIO_STEPPER_TRAY_PULSE: u8 = 23; // 16
pub const GPIO_STEPPER_TRAY_MOTOR_SPEED: u64 = 50;

pub const GPIO_RELAY_VACUUM: u8 = 20; // 38
pub const GPIO_RELAY_LIGHT: u8 = 16; // 36
pub const GPIO_KILL_SWITCH: u8 = 255; // ??

/*
u8 has the max value of 255
u16 has the max value of 65535
u32 has the max value of 4294967295
u64 has the max value of 18446744073709551615

i32 has the max value of 2147483647
*/
  