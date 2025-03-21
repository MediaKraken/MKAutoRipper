use sqlite::State;
use std::error::Error;
use std::path::Path;

#[non_exhaustive]
pub struct LogType;

impl LogType {
    pub const LOG_SNAPSHOT: u8 = 1;
    pub const LOG_STEPS_LEFT: u8 = 2;
    pub const LOG_STEPS_RIGHT: u8 = 3;
    pub const LOG_STEPS_DOWN: u8 = 4;
    pub const LOG_STEPS_UP: u8 = 5;
    pub const LOG_STEPS_FORWARD: u8 = 6;
    pub const LOG_STEPS_BACK: u8 = 7;
    pub const LOG_RELAY_LIGHT: u8 = 8;
    pub const LOG_RELAY_VACCUUM: u8 = 9;
    pub const LOG_APP_START: u8 = 10;
    pub const LOG_APP_EXIT: u8 = 11;
    pub const LOG_APP_RIPPING_START: u8 = 12;
    pub const LOG_APP_RIPPING_STOP: u8 = 13;
}

pub fn database_open() -> Result<sqlite::Connection, Box<dyn Error>> {
    let db = sqlite::open("pi_terminal.db").unwrap();
    let query = "CREATE TABLE IF NOT EXISTS totals \
            (steps_left INTEGER NOT NULL, \
            steps_right INTEGER NOT NULL, \
            steps_down INTEGER NOT NULL, \
            steps_up INTEGER NOT NULL, \
            steps_forward INTEGER NOT NULL, \
            steps_back INTEGER NOT NULL, \
            images_taken INTEGER NOT NULL, \
            cd_ripped INTEGER NOT NULL, \
            dvd_ripped INTEGER NOT NULL, \
            bray_ripped INTEGER NOT NULL, \
            uhd_ripped INTEGER NOT NULL, \
            hddvd_ripped INTEGER NOT NULL, \
            tracks_ripped INTEGER NOT NULL);";
    db.execute(query).unwrap();
    let query = "CREATE TABLE IF NOT EXISTS logs \
            (log_type INTEGER NOT NULL, \
            log_timestamp DATETIME NOT NULL)";
    db.execute(query).unwrap();
    Ok(db)
}

// cannot be called explicitly, drops when out of scope
// pub fn database_insert_clpose(
//     db: &sqlite::Connection,
// ) -> Result<(), Box<dyn Error>> {
//     db.drop();
//     Ok(())
// }

pub fn database_insert_logs(
    db: &sqlite::Connection,
    log_type: u8,
) -> Result<(), Box<dyn Error>> {
    let query = format!(
        "insert into logs (log_type, log_timestamp) \
        values ({}, CURRENT_TIMESTAMP);",
        log_type,
    );
    db.execute(query).unwrap();
    Ok(())
}

pub fn database_update_totals(
    db: &sqlite::Connection,
    total_type: &str,
    total_value: i32,
) -> Result<(), Box<dyn Error>> {
    let query = format!(
        "update totals set {} = {} += {}",
        total_type, total_type, total_value
    );
    db.execute(query).unwrap();
    Ok(())
}
