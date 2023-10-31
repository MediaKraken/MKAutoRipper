use lazy_static::lazy_static;

#[non_exhaustive]
pub struct DriveType;

impl DriveType {
    pub const CD: u16 = 1;
    pub const DVD: u16 = 2;
    pub const BRAY: u16 = 3;
    pub const UHD: u16 = 4;
    pub const HDDVD: u16 = 5;
}

lazy_static! {
    // these "locations" are ticks from 0 for the movement arm
    pub static ref INPUT_SPINDLE_LOCATIONS: [u16; 4] = [0, 200, 400, 600];
    pub static ref OUTPUT_SPINDLE_LOCATIONS: [u16; 4] = [1000, 1200, 1400, 1600];
}

/*
u8 has the max value of 255
u16 has the max value of 65535
u32 has the max value of 4294967295
u64 has the max value of 18446744073709551615
*/
lazy_static! {
    pub static ref DRIVE_LAYOUT: Vec<(u16, u16, u16, u16)> = vec![
        // bottom row of drives
        (0, DriveType::DVD, 100, 100),
        (1, DriveType::DVD, 100, 200),
        (2, DriveType::DVD, 100, 300),
        (3, DriveType::DVD, 100, 400),
        // 2nd row of drives
        (4, DriveType::DVD, 200, 100),
        (5, DriveType::DVD, 200, 200),
        (6, DriveType::DVD, 200, 300),
        (7, DriveType::DVD, 200, 400),
        // 3rd row of drives
        (8, DriveType::DVD, 300, 100),
        (9, DriveType::DVD, 300, 200),
        (10, DriveType::DVD, 300, 300),
        (11, DriveType::DVD, 300, 400),
        // 4th row of drives
        (12, DriveType::DVD, 400, 100),
        (13, DriveType::DVD, 400, 200),
        (14, DriveType::DVD, 400, 300),
        (15, DriveType::DVD, 400, 400),
        // 5th row of drives
        (16, DriveType::DVD, 500, 100),
        (17, DriveType::DVD, 500, 200),
        (18, DriveType::DVD, 500, 300),
        (19, DriveType::DVD, 500, 400),
        // 6th row of drives
        (20, DriveType::DVD, 600, 100),
        (21, DriveType::DVD, 600, 200),
        (22, DriveType::DVD, 600, 300),
        (23, DriveType::DVD, 600, 400),
        // top row of drives
        (24, DriveType::HDDVD, 700, 150),
        (25, DriveType::HDDVD, 700, 300),
        ];
}
