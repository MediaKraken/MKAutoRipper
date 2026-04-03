use sqlite::Connection;
use std::error::Error;

type DbResult<T> = Result<T, Box<dyn Error>>;

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

#[derive(Debug, Clone, Copy)]
pub enum TotalType {
    StepsLeft,
    StepsRight,
    StepsDown,
    StepsUp,
    StepsForward,
    StepsBack,
    ImagesTaken,
    CdRipped,
    DvdRipped,
    BrayRipped,
    UhdRipped,
    HddvdRipped,
    TracksRipped,
}

impl TotalType {
    fn as_column_name(self) -> &'static str {
        match self {
            TotalType::StepsLeft => "steps_left",
            TotalType::StepsRight => "steps_right",
            TotalType::StepsDown => "steps_down",
            TotalType::StepsUp => "steps_up",
            TotalType::StepsForward => "steps_forward",
            TotalType::StepsBack => "steps_back",
            TotalType::ImagesTaken => "images_taken",
            TotalType::CdRipped => "cd_ripped",
            TotalType::DvdRipped => "dvd_ripped",
            TotalType::BrayRipped => "bray_ripped",
            TotalType::UhdRipped => "uhd_ripped",
            TotalType::HddvdRipped => "hddvd_ripped",
            TotalType::TracksRipped => "tracks_ripped",
        }
    }
}

pub fn database_open() -> DbResult<Connection> {
    let db = sqlite::open("pi_terminal.db")?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS totals (
            steps_left INTEGER NOT NULL DEFAULT 0,
            steps_right INTEGER NOT NULL DEFAULT 0,
            steps_down INTEGER NOT NULL DEFAULT 0,
            steps_up INTEGER NOT NULL DEFAULT 0,
            steps_forward INTEGER NOT NULL DEFAULT 0,
            steps_back INTEGER NOT NULL DEFAULT 0,
            images_taken INTEGER NOT NULL DEFAULT 0,
            cd_ripped INTEGER NOT NULL DEFAULT 0,
            dvd_ripped INTEGER NOT NULL DEFAULT 0,
            bray_ripped INTEGER NOT NULL DEFAULT 0,
            uhd_ripped INTEGER NOT NULL DEFAULT 0,
            hddvd_ripped INTEGER NOT NULL DEFAULT 0,
            tracks_ripped INTEGER NOT NULL DEFAULT 0
        );",
    )?;

    db.execute(
        "CREATE TABLE IF NOT EXISTS logs (
            log_type INTEGER NOT NULL,
            log_timestamp DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );",
    )?;

    db.execute(
        "INSERT INTO totals (
            steps_left,
            steps_right,
            steps_down,
            steps_up,
            steps_forward,
            steps_back,
            images_taken,
            cd_ripped,
            dvd_ripped,
            bray_ripped,
            uhd_ripped,
            hddvd_ripped,
            tracks_ripped
        )
        SELECT 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        WHERE NOT EXISTS (SELECT 1 FROM totals);",
    )?;

    Ok(db)
}

pub fn database_insert_logs(db: &Connection, log_type: u8) -> DbResult<()> {
    let mut statement = db.prepare(
        "INSERT INTO logs (log_type, log_timestamp)
         VALUES (?, CURRENT_TIMESTAMP);",
    )?;

    statement.bind((1, log_type as i64))?;
    statement.next()?;
    Ok(())
}

pub fn database_update_totals(
    db: &Connection,
    total_type: TotalType,
    total_value: i32,
) -> DbResult<()> {
    let column = total_type.as_column_name();
    let query = format!("UPDATE totals SET {column} = {column} + ?;");

    let mut statement = db.prepare(query)?;
    statement.bind((1, total_value as i64))?;
    statement.next()?;
    Ok(())
}