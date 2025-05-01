use fltk::{
    app,
    app::*,
    button::*,
    enums::*,
    frame::*,
    group::*,
    menu::{Choice, MenuButton},
    prelude::*,
    window::*,
};
use fltk_table::{SmartTable, TableOpts};
use rppal::gpio::Gpio;
use rppal::pwm::{Channel, Pwm};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::uart::{Parity, Uart};
use serde_json::{json, Value};
use std::error::Error;
use std::path::Path;
use std::{cell::RefCell, rc::Rc};
use std::{fs, i32};
use tokio::time::{sleep, Duration};
use uuid::Uuid;

mod byte_size;
mod camera;
mod database;
mod gpio;
mod hardware_layout;
mod rabbitmq;
mod servo;
mod stepper;

pub fn find_steps_to_take(choice_string: i32) -> i32 {
    let mut steps_to_move: i32 = 1;
    match choice_string {
        1 => steps_to_move = 10,
        2 => steps_to_move = 100,
        3 => steps_to_move = 500,
        4 => steps_to_move = 1000,
        5 => steps_to_move = 5000,
        6 => steps_to_move = 10000,
        7 => steps_to_move = 25000,
        8 => steps_to_move = 50000,
        9 => steps_to_move = 100000,
        _ => {}
    }
    println!("{} {}", choice_string, steps_to_move);
    steps_to_move
}

#[tokio::main]
async fn main() {
    // drive number, allowed media types, horizontal, vertical, in use
    let mut drive_layout: Vec<(u16, Vec<&str>, i32, i32, bool)> = vec![
        // bottom row of drives
        (
            0,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[0],
            hardware_layout::DRIVE_ROW_LOCATIONS[0],
            false,
        ),
        (
            1,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[1],
            hardware_layout::DRIVE_ROW_LOCATIONS[0],
            false,
        ),
        (
            2,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[2],
            hardware_layout::DRIVE_ROW_LOCATIONS[0],
            false,
        ),
        (
            3,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[3],
            hardware_layout::DRIVE_ROW_LOCATIONS[0],
            false,
        ),
        // 2nd row of drives
        (
            4,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[0],
            hardware_layout::DRIVE_ROW_LOCATIONS[1],
            false,
        ),
        (
            5,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[1],
            hardware_layout::DRIVE_ROW_LOCATIONS[1],
            false,
        ),
        (
            6,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[2],
            hardware_layout::DRIVE_ROW_LOCATIONS[1],
            false,
        ),
        (
            7,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[3],
            hardware_layout::DRIVE_ROW_LOCATIONS[1],
            false,
        ),
        // 3rd row of drives
        (
            8,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[0],
            hardware_layout::DRIVE_ROW_LOCATIONS[2],
            false,
        ),
        (
            9,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[1],
            hardware_layout::DRIVE_ROW_LOCATIONS[2],
            false,
        ),
        (
            10,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[2],
            hardware_layout::DRIVE_ROW_LOCATIONS[2],
            false,
        ),
        (
            11,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[3],
            hardware_layout::DRIVE_ROW_LOCATIONS[2],
            false,
        ),
        // 4th row of drives
        (
            12,
            vec![hardware_layout::DRIVETYPE_BRAY],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[0],
            hardware_layout::DRIVE_ROW_LOCATIONS[3],
            false,
        ),
        (
            13,
            vec![hardware_layout::DRIVETYPE_BRAY],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[1],
            hardware_layout::DRIVE_ROW_LOCATIONS[3],
            false,
        ),
        (
            14,
            vec![hardware_layout::DRIVETYPE_BRAY],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[2],
            hardware_layout::DRIVE_ROW_LOCATIONS[3],
            false,
        ),
        (
            15,
            vec![hardware_layout::DRIVETYPE_BRAY],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[3],
            hardware_layout::DRIVE_ROW_LOCATIONS[3],
            false,
        ),
        // 5th row of drives
        (
            16,
            vec![hardware_layout::DRIVETYPE_BRAY],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[0],
            hardware_layout::DRIVE_ROW_LOCATIONS[4],
            false,
        ),
        (
            17,
            vec![hardware_layout::DRIVETYPE_BRAY],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[1],
            hardware_layout::DRIVE_ROW_LOCATIONS[4],
            false,
        ),
        (
            18,
            vec![hardware_layout::DRIVETYPE_BRAY],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[2],
            hardware_layout::DRIVE_ROW_LOCATIONS[4],
            false,
        ),
        (
            19,
            vec![hardware_layout::DRIVETYPE_BRAY],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[3],
            hardware_layout::DRIVE_ROW_LOCATIONS[4],
            false,
        ),
        // 6th row of drives
        (
            20,
            vec![hardware_layout::DRIVETYPE_UHD],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[0],
            hardware_layout::DRIVE_ROW_LOCATIONS[5],
            false,
        ),
        (
            21,
            vec![hardware_layout::DRIVETYPE_UHD],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[1],
            hardware_layout::DRIVE_ROW_LOCATIONS[5],
            false,
        ),
        (
            22,
            vec![hardware_layout::DRIVETYPE_UHD],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[2],
            hardware_layout::DRIVE_ROW_LOCATIONS[5],
            false,
        ),
        (
            23,
            vec![hardware_layout::DRIVETYPE_UHD],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[3],
            hardware_layout::DRIVE_ROW_LOCATIONS[5],
            false,
        ),
        // top row of drives - hddvd
        (
            24,
            vec![hardware_layout::DRIVETYPE_HDDVD],
            hardware_layout::DRIVE_COLUMN_LOCATIONS[4],
            hardware_layout::DRIVE_ROW_LOCATIONS[6],
            false,
        ),
    ];
    // connect to database
    let db_pool = database::database_open().unwrap();

    // connect to rabbitmq
    let (rabbit_connection, rabbit_channel) =
        rabbitmq::rabbitmq_connect("mkterminal").await.unwrap();
    let mut rabbit_consumer = rabbitmq::rabbitmq_consumer("mkterminal", &rabbit_channel)
        .await
        .unwrap();
    let mut hard_stop: bool = false;
    let mut gpio_relay_vacuum_on: bool = false;
    let app = app::App::default();

    let position_horizontal = Rc::new(RefCell::new(0));
    let position_vertical = Rc::new(RefCell::new(0));
    let position_camera_tray = Rc::new(RefCell::new(0));

    let mut win = Window::new(0, 0, 800, 500, "pi_terminal for autoripper");

    let mut container_spindle = Pack::new(10, 25, 300, 35, "Spindle Type");
    // setup control for spindle media
    let mut choice_spindle_1_media_type = Choice::new(20, 20, 120, 30, None);
    choice_spindle_1_media_type.add_choice(hardware_layout::DRIVETYPE_NONE);
    choice_spindle_1_media_type.add_choice(hardware_layout::DRIVETYPE_CD);
    choice_spindle_1_media_type.add_choice(hardware_layout::DRIVETYPE_DVD);
    choice_spindle_1_media_type.add_choice(hardware_layout::DRIVETYPE_BRAY);
    choice_spindle_1_media_type.add_choice(hardware_layout::DRIVETYPE_UHD);
    choice_spindle_1_media_type.add_choice(hardware_layout::DRIVETYPE_HDDVD);
    choice_spindle_1_media_type.set_value(0);

    // setup control for spindle media
    let mut choice_spindle_2_media_type = Choice::new(20, 175, 120, 30, None);
    choice_spindle_2_media_type.add_choice(hardware_layout::DRIVETYPE_NONE);
    choice_spindle_2_media_type.add_choice(hardware_layout::DRIVETYPE_CD);
    choice_spindle_2_media_type.add_choice(hardware_layout::DRIVETYPE_DVD);
    choice_spindle_2_media_type.add_choice(hardware_layout::DRIVETYPE_BRAY);
    choice_spindle_2_media_type.add_choice(hardware_layout::DRIVETYPE_UHD);
    choice_spindle_2_media_type.add_choice(hardware_layout::DRIVETYPE_HDDVD);
    choice_spindle_2_media_type.set_value(0);

    // setup control for spindle media
    let mut choice_spindle_3_media_type = Choice::new(20, 280, 120, 30, None);
    choice_spindle_3_media_type.add_choice(hardware_layout::DRIVETYPE_NONE);
    choice_spindle_3_media_type.add_choice(hardware_layout::DRIVETYPE_CD);
    choice_spindle_3_media_type.add_choice(hardware_layout::DRIVETYPE_DVD);
    choice_spindle_3_media_type.add_choice(hardware_layout::DRIVETYPE_BRAY);
    choice_spindle_3_media_type.add_choice(hardware_layout::DRIVETYPE_UHD);
    choice_spindle_3_media_type.add_choice(hardware_layout::DRIVETYPE_HDDVD);
    choice_spindle_3_media_type.set_value(0);

    container_spindle.end();
    container_spindle.set_frame(FrameType::BorderFrame);
    container_spindle.set_color(Color::Black);
    container_spindle.set_type(PackType::Horizontal);

    let mut container_status = Pack::new(10, 90, 320, 375, "Status");

    let mut status_table = SmartTable::default()
        .with_size(320, 350)
        .center_of_parent()
        .with_opts(TableOpts {
            rows: 24,
            cols: 3,
            editable: false,
            ..Default::default()
        });

    container_status.end();
    container_status.set_frame(FrameType::BorderFrame);
    container_status.set_color(Color::Black);
    container_status.set_type(PackType::Horizontal);
    let mut container_info = Pack::new(345, 90, 225, 150, "Info");

    let mut info_table = SmartTable::default()
        .with_size(225, 150)
        .center_of_parent()
        .with_opts(TableOpts {
            rows: 5,
            cols: 2,
            editable: false,
            ..Default::default()
        });
    info_table.set_col_width(1, 140);
    info_table.set_row_header_width(0);
    info_table.set_cell_value(0, 0, "Model");
    if Path::new("/sys/firmware/devicetree/base/model").exists() {
        let pi_model = fs::read_to_string("/sys/firmware/devicetree/base/model").unwrap();
        info_table.set_cell_value(0, 1, &pi_model);
    } else {
        info_table.set_cell_value(0, 1, "Desktop");
    }
    info_table.set_cell_value(1, 0, "Memory");
    info_table.set_cell_value(
        1,
        1,
        &byte_size::mk_lib_common_bytesize(sys_info::mem_info().unwrap().total * 1000).unwrap(),
    );
    info_table.set_cell_value(2, 0, "Disk");
    info_table.set_cell_value(
        2,
        1,
        &byte_size::mk_lib_common_bytesize(sys_info::disk_info().unwrap().total * 1000).unwrap(),
    );
    info_table.set_cell_value(3, 0, "Camera");
    info_table.set_cell_value(3, 1, "N/A");
    info_table.set_cell_value(4, 0, "OS");
    let os_release_info = sys_info::linux_os_release().unwrap();
    info_table.set_cell_value(
        4,
        1,
        &format!(
            "{:?} {:?}",
            os_release_info.name, os_release_info.version_id
        ),
    );

    container_info.end();
    container_info.set_frame(FrameType::BorderFrame);
    container_info.set_color(Color::Black);
    container_info.set_type(PackType::Horizontal);

    let mut container_action = Pack::new(345, 265, 225, 35, "Action Type");
    let mut container_action_type = Choice::new(20, 20, 225, 35, None);
    container_action_type.add_choice(
        "1 Step|10 Steps|100 Steps|500 Steps|1,000 Steps|5,000 Steps|10,000 Steps|25,000 Steps|50,000 Steps|100,000 Steps|Input One|Input Two|Input Three|Output One|Output Two|Output Three|Output FourDrive Column One|Drive Column Two|Drive Column Three|Drive Column Four|Column Camera|Column HDDVD|Drive Row One|Drive Row Two|Drive Row Three|Drive Row Four|Row Camera|Row HDDVD|Horizontal to 0|Vertical to 0|Camera to 0");
    container_action_type.set_value(0);
    container_action.end();
    container_action.set_frame(FrameType::BorderFrame);
    container_action.set_color(Color::Black);
    container_action.set_type(PackType::Horizontal);
    let mut button_execute_combobox = Button::new(345, 310, 225, 40, "Execute Combobox");

    let mut container_position = Pack::new(590, 25, 200, 40, "Position - step(s)");

    let mut frame_position_horizontal = Frame::default().with_size(40, 20).with_label("Horiz: 0");
    let mut frame_position_vertical = Frame::default().with_size(40, 20).with_label("Vert: 0");
    let mut frame_position_camera_tray = Frame::default().with_size(40, 20).with_label("Tray: 0");

    container_position.end();
    container_position.set_frame(FrameType::BorderFrame);
    container_position.set_color(Color::Black);
    container_position.set_type(PackType::Vertical);

    // move the arms around
    let mut button_zero = Button::new(400, 15, 150, 50, "Zero Everything");
    // main track
    let mut button_left = Button::new(590, 100, 50, 50, "L");
    let mut button_left_full_rotation = Button::new(640, 100, 50, 50, "L F");
    let mut button_right = Button::new(700, 100, 50, 50, "R");
    let mut button_right_full_rotation = Button::new(750, 100, 50, 50, "R F");
    // pickup arm
    let mut button_up = Button::new(590, 175, 50, 50, "U");
    let mut button_up_full_rotation = Button::new(640, 175, 50, 50, "U F");
    let mut button_down = Button::new(700, 175, 50, 50, "D");
    let mut button_down_full_rotation = Button::new(750, 175, 50, 50, "D F");
    // camera tray
    let mut button_back = Button::new(590, 250, 50, 50, "B");
    let mut button_back_full_rotation = Button::new(640, 250, 50, 50, "B F");
    let mut button_forward = Button::new(700, 250, 50, 50, "F");
    let mut button_forward_full_rotation = Button::new(750, 250, 50, 50, "F F");

    // activate equipment
    let mut button_vacuum = Button::new(600, 320, 80, 50, "Vacuum");
    let mut button_snapshot = Button::new(700, 320, 80, 50, "Snapshot");

    // start/stop ripping
    let mut button_start = Button::new(500, 390, 150, 60, "Start Ripping!");
    let mut button_stop = Button::new(650, 390, 150, 60, "Stop!");

    // exit
    let mut button_exit = Button::new(345, 390, 150, 60, "EXIT!");

    win.end();
    win.show();

    button_execute_combobox.set_callback({
        // horizontal
        let position_horizontal = position_horizontal.clone();
        let mut frame_position_horizontal = frame_position_horizontal.clone();
        let position_horizontal_int = *(position_horizontal.borrow());
        // vertical
        let position_vertical = position_vertical.clone();
        let mut frame_position_vertical = frame_position_vertical.clone();
        let position_vertical_int = *(position_vertical.borrow());
        // camera tray
        let position_camera_tray = position_camera_tray.clone();
        let mut frame_position_camera_tray = frame_position_camera_tray.clone();
        let position_camera_tray_int = *(position_camera_tray.borrow());

        let choice_string = container_action_type.clone();
        move |_| {
            let mut move_clockwise = false;
            let mut steps_to_move: i32 = 0;
            let mut hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT;
            let choice_string_string = choice_string.choice().unwrap();
            match choice_string_string.as_str() {
                "Input One" => {
                    if position_horizontal_int <= hardware_layout::INPUT_SPINDLE_LOCATIONS[0] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::INPUT_SPINDLE_LOCATIONS[0];
                    } else {
                        steps_to_move =
                            hardware_layout::INPUT_SPINDLE_LOCATIONS[0] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Input Two" => {
                    if position_horizontal_int <= hardware_layout::INPUT_SPINDLE_LOCATIONS[1] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::INPUT_SPINDLE_LOCATIONS[1];
                    } else {
                        steps_to_move =
                            hardware_layout::INPUT_SPINDLE_LOCATIONS[1] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Input Three" => {
                    if position_horizontal_int <= hardware_layout::INPUT_SPINDLE_LOCATIONS[2] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::INPUT_SPINDLE_LOCATIONS[2];
                    } else {
                        steps_to_move =
                            hardware_layout::INPUT_SPINDLE_LOCATIONS[2] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Output One" => {
                    if position_horizontal_int <= hardware_layout::OUTPUT_SPINDLE_LOCATIONS[0] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::OUTPUT_SPINDLE_LOCATIONS[0];
                    } else {
                        steps_to_move =
                            hardware_layout::OUTPUT_SPINDLE_LOCATIONS[0] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Output Two" => {
                    if position_horizontal_int <= hardware_layout::OUTPUT_SPINDLE_LOCATIONS[1] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::OUTPUT_SPINDLE_LOCATIONS[1];
                    } else {
                        steps_to_move =
                            hardware_layout::OUTPUT_SPINDLE_LOCATIONS[1] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Output Three" => {
                    if position_horizontal_int <= hardware_layout::OUTPUT_SPINDLE_LOCATIONS[2] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::OUTPUT_SPINDLE_LOCATIONS[2];
                    } else {
                        steps_to_move =
                            hardware_layout::OUTPUT_SPINDLE_LOCATIONS[2] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Output Four" => {
                    if position_horizontal_int <= hardware_layout::OUTPUT_SPINDLE_LOCATIONS[3] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::OUTPUT_SPINDLE_LOCATIONS[3];
                    } else {
                        steps_to_move =
                            hardware_layout::OUTPUT_SPINDLE_LOCATIONS[3] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Drive Column One" => {
                    if position_horizontal_int <= hardware_layout::DRIVE_COLUMN_LOCATIONS[0] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::DRIVE_COLUMN_LOCATIONS[0];
                    } else {
                        steps_to_move =
                            hardware_layout::DRIVE_COLUMN_LOCATIONS[0] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Drive Column Two" => {
                    if position_horizontal_int <= hardware_layout::DRIVE_COLUMN_LOCATIONS[1] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::DRIVE_COLUMN_LOCATIONS[1];
                    } else {
                        steps_to_move =
                            hardware_layout::DRIVE_COLUMN_LOCATIONS[1] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Drive Column Three" => {
                    if position_horizontal_int <= hardware_layout::DRIVE_COLUMN_LOCATIONS[2] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::DRIVE_COLUMN_LOCATIONS[2];
                    } else {
                        steps_to_move =
                            hardware_layout::DRIVE_COLUMN_LOCATIONS[2] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Drive Column Four" => {
                    if position_horizontal_int <= hardware_layout::DRIVE_COLUMN_LOCATIONS[3] {
                        steps_to_move =
                            position_horizontal_int - hardware_layout::DRIVE_COLUMN_LOCATIONS[3];
                    } else {
                        steps_to_move =
                            hardware_layout::DRIVE_COLUMN_LOCATIONS[3] - position_horizontal_int;
                        move_clockwise = true;
                        hard_stop_pin = hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
                    }
                }
                "Column Camera" => {
                    if position_horizontal_int <= hardware_layout::INPUT_SPINDLE_LOCATIONS[0] {
                    } else {
                    }
                }
                "Column HDDVD" => {
                    if position_horizontal_int <= hardware_layout::INPUT_SPINDLE_LOCATIONS[0] {
                    } else {
                    }
                }
                "Drive Row One" => {}
                "Drive Row Two" => {}
                "Drive Row Three" => {}
                "Drive Row Four" => {}
                "Row Camera" => {}
                "Row HDDVD" => {}
                "Horizontal to 0" => {
                    *position_horizontal.borrow_mut() = 0;
                    frame_position_horizontal.set_label(&"Horiz: 0");
                }
                "Vertical to 0" => {
                    *position_vertical.borrow_mut() = 0;
                    frame_position_vertical.set_label(&"Vert: 0");
                }
                "Camera to 0" => {
                    *position_camera_tray.borrow_mut() = 0;
                    frame_position_camera_tray.set_label(&"Tray: 0");
                }
                _ => println!("ERROR! Bad Action Type"),
            }
            if steps_to_move != 0 {
                let steps_taken = stepper::gpio_stepper_move(
                    steps_to_move.abs(),
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                    hard_stop_pin,
                    move_clockwise,
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
                );
                *position_horizontal.borrow_mut() += steps_taken.unwrap();
                frame_position_horizontal.set_label(&format!(
                    "Horiz: {}",
                    &position_horizontal.borrow().to_string()
                ));
            }
        }
    });

    // position button click processing
    button_right.set_callback({
        let position_horizontal = position_horizontal.clone();
        let mut frame_position_horizontal = frame_position_horizontal.clone();
        let action_type = container_action_type.clone();
        move |_| {
            let steps_to_move: i32 = find_steps_to_take(action_type.value());
            let steps_taken = stepper::gpio_stepper_move(
                steps_to_move,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                false,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
            );
            *position_horizontal.borrow_mut() += steps_taken.unwrap();
            frame_position_horizontal.set_label(&format!(
                "Horiz: {}",
                &position_horizontal.borrow().to_string()
            ));
        }
    });

    button_right_full_rotation.set_callback({
        let position_horizontal = position_horizontal.clone();
        let mut frame_position_horizontal = frame_position_horizontal.clone();
        move |_| {
            let steps_taken = stepper::gpio_stepper_move(
                i32::MAX,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                false,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
            );
            *position_horizontal.borrow_mut() += steps_taken.unwrap();
            frame_position_horizontal.set_label(&format!(
                "Horiz: {}",
                &position_horizontal.borrow().to_string()
            ));
        }
    });

    button_left.set_callback({
        let position_horizontal = position_horizontal.clone();
        let mut frame_position_horizontal = frame_position_horizontal.clone();
        let action_type = container_action_type.clone();
        move |_| {
            let steps_to_move: i32 = find_steps_to_take(action_type.value());
            let steps_taken = stepper::gpio_stepper_move(
                steps_to_move,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
                true,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
            );
            *position_horizontal.borrow_mut() -= steps_taken.unwrap();
            frame_position_horizontal
                .set_label(&format!("Horiz: {}", &position_horizontal.borrow()));
        }
    });

    button_left_full_rotation.set_callback({
        let position_horizontal = position_horizontal.clone();
        let mut frame_position_horizontal = frame_position_horizontal.clone();
        move |_| {
            let steps_taken = stepper::gpio_stepper_move(
                i32::MAX,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
                true,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
            );
            *position_horizontal.borrow_mut() -= steps_taken.unwrap();
            frame_position_horizontal
                .set_label(&format!("Horiz: {}", &position_horizontal.borrow()));
        }
    });

    button_up.set_callback({
        let position_vertical = position_vertical.clone();
        let mut frame_position_vertical = frame_position_vertical.clone();
        let action_type = container_action_type.clone();
        move |_| {
            let steps_to_move: i32 = find_steps_to_take(action_type.value());
            let steps_taken = stepper::gpio_stepper_move(
                steps_to_move,
                hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                true,
                hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
            );
            *position_vertical.borrow_mut() += steps_taken.unwrap();
            frame_position_vertical.set_label(&format!(
                "Vert: {}",
                &position_vertical.borrow().to_string()
            ));
        }
    });

    button_up_full_rotation.set_callback({
        let position_vertical = position_vertical.clone();
        let mut frame_position_vertical = frame_position_vertical.clone();
        move |_| {
            let steps_taken = stepper::gpio_stepper_move(
                i32::MAX,
                hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                true,
                hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
            );
            *position_vertical.borrow_mut() += steps_taken.unwrap();
            frame_position_vertical.set_label(&format!(
                "Vert: {}",
                &position_vertical.borrow().to_string()
            ));
        }
    });

    button_down.set_callback({
        let position_vertical = position_vertical.clone();
        let mut frame_position_vertical = frame_position_vertical.clone();
        let action_type = container_action_type.clone();
        move |_| {
            let steps_to_move: i32 = find_steps_to_take(action_type.value());
            let steps_taken = stepper::gpio_stepper_move(
                steps_to_move,
                hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM,
                false,
                hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
            );
            *position_vertical.borrow_mut() -= steps_taken.unwrap();
            frame_position_vertical.set_label(&format!("Vert: {}", &position_vertical.borrow()));
        }
    });

    button_down_full_rotation.set_callback({
        let position_vertical = position_vertical.clone();
        let mut frame_position_vertical = frame_position_vertical.clone();
        move |_| {
            let steps_taken = stepper::gpio_stepper_move(
                i32::MAX,
                hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM,
                false,
                hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
            );
            *position_vertical.borrow_mut() -= steps_taken.unwrap();
            frame_position_vertical.set_label(&format!("Vert: {}", &position_vertical.borrow()));
        }
    });

    button_back.set_callback({
        let position_camera_tray = position_camera_tray.clone();
        let mut frame_position_camera_tray = frame_position_camera_tray.clone();
        let action_type = container_action_type.clone();
        move |_| {
            let steps_to_move: i32 = find_steps_to_take(action_type.value());
            let steps_taken = stepper::gpio_stepper_move(
                steps_to_move,
                hardware_layout::GPIO_STEPPER_TRAY_PULSE,
                hardware_layout::GPIO_STEPPER_TRAY_DIRECTION,
                hardware_layout::GPIO_STEPPER_TRAY_END_STOP_BACK,
                true,
                hardware_layout::GPIO_STEPPER_TRAY_MOTOR_SPEED,
            );
            *position_camera_tray.borrow_mut() += steps_taken.unwrap();
            frame_position_camera_tray.set_label(&format!(
                "Tray: {}",
                &position_camera_tray.borrow().to_string()
            ));
        }
    });

    button_back_full_rotation.set_callback({
        let position_camera_tray = position_camera_tray.clone();
        let mut frame_position_camera_tray = frame_position_camera_tray.clone();
        move |_| {
            let steps_taken = stepper::gpio_stepper_move(
                i32::MAX,
                hardware_layout::GPIO_STEPPER_TRAY_PULSE,
                hardware_layout::GPIO_STEPPER_TRAY_DIRECTION,
                hardware_layout::GPIO_STEPPER_TRAY_END_STOP_BACK,
                true,
                hardware_layout::GPIO_STEPPER_TRAY_MOTOR_SPEED,
            );
            *position_camera_tray.borrow_mut() += steps_taken.unwrap();
            frame_position_camera_tray.set_label(&format!(
                "Tray: {}",
                &position_camera_tray.borrow().to_string()
            ));
        }
    });

    button_forward.set_callback({
        let position_camera_tray = position_camera_tray.clone();
        let mut frame_position_camera_tray = frame_position_camera_tray.clone();
        let action_type = container_action_type.clone();
        move |_| {
            let steps_to_move: i32 = find_steps_to_take(action_type.value());
            let steps_taken = stepper::gpio_stepper_move(
                steps_to_move,
                hardware_layout::GPIO_STEPPER_TRAY_PULSE,
                hardware_layout::GPIO_STEPPER_TRAY_DIRECTION,
                hardware_layout::GPIO_STEPPER_TRAY_END_STOP_FRONT,
                false,
                hardware_layout::GPIO_STEPPER_TRAY_MOTOR_SPEED,
            );
            *position_camera_tray.borrow_mut() -= steps_taken.unwrap();
            frame_position_camera_tray
                .set_label(&format!("Tray: {}", &position_camera_tray.borrow()));
        }
    });

    button_forward_full_rotation.set_callback({
        let position_camera_tray: Rc<RefCell<i32>> = position_camera_tray.clone();
        let mut frame_position_camera_tray = frame_position_camera_tray.clone();
        move |_| {
            let steps_taken = stepper::gpio_stepper_move(
                i32::MAX,
                hardware_layout::GPIO_STEPPER_TRAY_PULSE,
                hardware_layout::GPIO_STEPPER_TRAY_DIRECTION,
                hardware_layout::GPIO_STEPPER_TRAY_END_STOP_FRONT,
                false,
                hardware_layout::GPIO_STEPPER_TRAY_MOTOR_SPEED,
            );
            *position_camera_tray.borrow_mut() -= steps_taken.unwrap();
            frame_position_camera_tray
                .set_label(&format!("Tray: {}", &position_camera_tray.borrow()));
        }
    });

    button_vacuum.set_callback(move |_| {
        // toggle vacuum
        gpio_relay_vacuum_on = !gpio_relay_vacuum_on;
        let _result = gpio::gpio_set_pin(gpio_relay_vacuum_on, hardware_layout::GPIO_RELAY_VACUUM);
        let _result =
            database::database_insert_logs(&db_pool, database::LogType::LOG_RELAY_VACCUUM);
    });

    button_snapshot.set_callback(move |_| {
        let _result = gpio::gpio_set_pin(true, hardware_layout::GPIO_RELAY_LIGHT);
        // let _result = database::database_insert_logs(
        //     &db_pool,
        //     database::LogType::LOG_RELAY_LIGHT,
        // );
        let _result = camera::camera_take_image("demo.png");
        // let _result = database::database_insert_logs(
        //     &db_pool,
        //     database::LogType::LOG_SNAPSHOT,
        // );
        //let _result = database::database_update_totals(&db_pool, "images_taken", 1);
        let _result = gpio::gpio_set_pin(false, hardware_layout::GPIO_RELAY_LIGHT);
        // let _result = database::database_insert_logs(
        //     &db_pool,
        //     database::LogType::LOG_RELAY_LIGHT,
        // );
    });

    button_zero.set_callback(
        // "home" the gantries
        {
            let position_camera_tray = position_camera_tray.clone();
            let mut frame_position_camera_tray = frame_position_camera_tray.clone();

            let position_vertical = position_vertical.clone();
            let mut frame_position_vertical = frame_position_vertical.clone();

            let position_horizontal = position_horizontal.clone();
            let mut frame_position_horizontal = frame_position_horizontal.clone();

            move |_| {
                // retract camera tray
                let steps_taken = stepper::gpio_stepper_move(
                    i32::MAX,
                    hardware_layout::GPIO_STEPPER_TRAY_PULSE,
                    hardware_layout::GPIO_STEPPER_TRAY_DIRECTION,
                    hardware_layout::GPIO_STEPPER_TRAY_END_STOP_BACK,
                    false,
                    hardware_layout::GPIO_STEPPER_TRAY_MOTOR_SPEED,
                );
                *position_camera_tray.borrow_mut() = 0;
                let steps_taken = stepper::gpio_stepper_move(
                    50, // TODO real number
                    hardware_layout::GPIO_STEPPER_TRAY_PULSE,
                    hardware_layout::GPIO_STEPPER_TRAY_DIRECTION,
                    hardware_layout::GPIO_STEPPER_TRAY_END_STOP_FRONT,
                    true,
                    hardware_layout::GPIO_STEPPER_TRAY_MOTOR_SPEED,
                );
                *position_camera_tray.borrow_mut() += steps_taken.unwrap();
                frame_position_camera_tray
                    .set_label(&format!("Tray: {}", &position_camera_tray.borrow()));

                // move loader to top
                let steps_taken = stepper::gpio_stepper_move(
                    i32::MAX,
                    hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                    hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                    hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                    true,
                    hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
                );
                *position_vertical.borrow_mut() = 0;
                let steps_taken = stepper::gpio_stepper_move(
                    50, // TODO real number
                    hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                    hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                    hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM,
                    false,
                    hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
                );
                *position_vertical.borrow_mut() -= steps_taken.unwrap();
                frame_position_vertical
                    .set_label(&format!("Vert: {}", &position_vertical.borrow()));

                // move gantry to far left
                let steps_taken = stepper::gpio_stepper_move(
                    i32::MAX,
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
                    false,
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
                );
                *position_horizontal.borrow_mut() = 0;
                let steps_taken = stepper::gpio_stepper_move(
                    i32::MAX,
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                    true,
                    hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
                );
                *position_horizontal.borrow_mut() += steps_taken.unwrap();
                frame_position_horizontal
                    .set_label(&format!("Horiz: {}", &position_horizontal.borrow()));
            }
        },
    );

    button_start.set_callback(move |_| {
        // TODO start ripping media
    });

    button_stop.set_callback(move |_| {
        // TODO stop the system immediately
    });

    button_exit.set_callback(move |_| {
        std::process::exit(0);
    });

    app.run().unwrap();

    // launch thread to do the actual processing of the discs
    //    let _handle_tmdb = tokio::spawn(async move {=
    let mut initial_start = true;
    let mut spindle_one_media_left = false;
    let mut spindle_two_media_left = false;
    let mut spindle_three_media_left = false;
    if choice_spindle_1_media_type.choice().unwrap().as_str() != hardware_layout::DRIVETYPE_NONE {
        spindle_one_media_left = true;
    }
    if choice_spindle_2_media_type.choice().unwrap().as_str() != hardware_layout::DRIVETYPE_NONE {
        spindle_two_media_left = true;
    }
    if choice_spindle_3_media_type.choice().unwrap().as_str() != hardware_layout::DRIVETYPE_NONE {
        spindle_three_media_left = true;
    }
    loop {
        // check for HARD stop
        if hard_stop {
            break;
        }
        // grab message from rabbitmq if one is available
        let msg = rabbit_consumer.recv().await;
        if let payload = msg.unwrap().content {
            let json_message: Value =
                serde_json::from_str(&String::from_utf8_lossy(&payload.unwrap())).unwrap();
            if json_message["Type"] == "done" {
                // TODO position horizontal
                // TODO position vertical
                // send eject rabbitmq message
                rabbitmq::rabbitmq_publish(
                    rabbit_channel.clone(),
                    json_message["instance"].as_str().unwrap(),
                    "{{'Type': 'eject'}}".to_string(),
                )
                .await
                .unwrap();
                // allow time to umount/eject
                sleep(Duration::from_secs(10)).await;
                // TODO place arm for media pickup
                // pick up media
                let _result = gpio::gpio_set_pin(true, hardware_layout::GPIO_RELAY_VACUUM);
                // send rabbitmq message to close tray
                rabbitmq::rabbitmq_publish(
                    rabbit_channel.clone(),
                    json_message["instance"].as_str().unwrap(),
                    "{{'Type': 'close'}}".to_string(),
                )
                .await
                .unwrap();
                // TODO set the drive back to empty
                // TODO raise arm if needed
                // TODO place media at exit spindle
                // TODO place media at drop height
                // drop media
                let _result = gpio::gpio_set_pin(false, hardware_layout::GPIO_RELAY_VACUUM);
            }
        }
        // process next disc
        if spindle_one_media_left {
            // determine usable drive
            for individual_drive_ndx in 0..drive_layout.len() {
                // check to see if the drive is in use
                if drive_layout[individual_drive_ndx].4 == false {
                    // check to see if the drive has the proper media capability
                    if drive_layout[individual_drive_ndx]
                        .1
                        .contains(&choice_spindle_1_media_type.choice().unwrap().as_str())
                    {
                        // raise the vertical to above the spindle
                        let steps_taken = stepper::gpio_stepper_move(
                            hardware_layout::SPINDLE_HEIGHT,
                            hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                            hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                            hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                            true,
                            hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
                        );
                        *position_vertical.borrow_mut() += steps_taken.unwrap();
                        // zero out the horizontal
                        let steps_taken = stepper::gpio_stepper_move(
                            i32::MAX,
                            hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                            hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                            hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
                            false,
                            hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
                        );
                        *position_horizontal.borrow_mut() -= steps_taken.unwrap();
                        // move horizontal to spindle one location
                        let steps_taken = stepper::gpio_stepper_move(
                            hardware_layout::INPUT_SPINDLE_LOCATIONS[0],
                            hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                            hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                            hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                            true,
                            hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
                        );
                        *position_horizontal.borrow_mut() += steps_taken.unwrap();
                        // lower vertical to stop switch on assembly
                        let steps_taken = stepper::gpio_stepper_move(
                            5000, // TODO put in real number
                            hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                            hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                            hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_ASSEMBLY,
                            false,
                            hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
                        );
                        *position_vertical.borrow_mut() -= steps_taken.unwrap();
                        // TODO if spindle stop switch triggers, turn off media switch and break
                        if *position_vertical.borrow() >= 0 {
                            // pick up media from spindle
                            let _result: Result<(), Box<dyn Error>> =
                                gpio::gpio_set_pin(true, hardware_layout::GPIO_RELAY_VACUUM);
                            // raise vertical to the camera level
                            let steps_taken = stepper::gpio_stepper_move(
                                hardware_layout::CAMERA_LOCATION.1,
                                hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                                hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                                hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                                true,
                                hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
                            );
                            *position_vertical.borrow_mut() += steps_taken.unwrap();
                            // move horizontal to camera
                            let steps_taken = stepper::gpio_stepper_move(
                                hardware_layout::CAMERA_LOCATION.0,
                                hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                                hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                                hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                                true,
                                hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
                            );
                            *position_horizontal.borrow_mut() += steps_taken.unwrap();

                            // move camera plate out
                            let steps_taken = stepper::gpio_stepper_move(
                                hardware_layout::CAMERA_PLATE_STEPS,
                                hardware_layout::GPIO_STEPPER_TRAY_PULSE,
                                hardware_layout::GPIO_STEPPER_TRAY_DIRECTION,
                                hardware_layout::GPIO_STEPPER_TRAY_END_STOP_FRONT,
                                true,
                                hardware_layout::GPIO_STEPPER_TRAY_MOTOR_SPEED,
                            );
                            *position_camera_tray.borrow_mut() += steps_taken.unwrap();
                            // turn on led
                            let _result: Result<(), Box<dyn Error>> =
                                gpio::gpio_set_pin(true, hardware_layout::GPIO_RELAY_LIGHT);
                            // generate path/name to use
                            let path_photo_name = Uuid::now_v7().to_string();
                            let _result =
                                fs::create_dir_all(format!("/nfsmount/{}", path_photo_name));
                            // take photo
                            let _result = camera::camera_take_image(
                                format!("/nfsmount/{}/{}.png", path_photo_name, path_photo_name)
                                    .as_str(),
                            );
                            // turn off led
                            let _result: Result<(), Box<dyn Error>> =
                                gpio::gpio_set_pin(false, hardware_layout::GPIO_RELAY_LIGHT);
                            // move camera plate in
                            let steps_taken = stepper::gpio_stepper_move(
                                hardware_layout::CAMERA_PLATE_STEPS,
                                hardware_layout::GPIO_STEPPER_TRAY_PULSE,
                                hardware_layout::GPIO_STEPPER_TRAY_DIRECTION,
                                hardware_layout::GPIO_STEPPER_TRAY_END_STOP_FRONT,
                                false,
                                hardware_layout::GPIO_STEPPER_TRAY_MOTOR_SPEED,
                            );
                            *position_camera_tray.borrow_mut() -= steps_taken.unwrap();
                            // move horizontal to drive row
                            let current_position: i32 = *(position_horizontal.borrow());
                            if current_position < drive_layout[individual_drive_ndx].2 {
                                // move arm right
                                let steps_to_take =
                                    drive_layout[individual_drive_ndx].2 - current_position;
                                let steps_taken = stepper::gpio_stepper_move(
                                    steps_to_take,
                                    hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                                    hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                                    hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                                    true,
                                    hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
                                );
                                *position_horizontal.borrow_mut() -= steps_taken.unwrap();
                            } else {
                                // move arm left
                                let steps_to_take =
                                    drive_layout[individual_drive_ndx].2 - current_position;
                                let steps_taken = stepper::gpio_stepper_move(
                                    steps_to_take,
                                    hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                                    hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                                    hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
                                    false,
                                    hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
                                );
                                *position_horizontal.borrow_mut() += steps_taken.unwrap();
                            }
                            // move vertical to above the drive tray
                            let current_position: i32 = *(position_vertical.borrow());
                            if current_position < drive_layout[individual_drive_ndx].2 {
                                // move arm up
                                let steps_to_take =
                                    drive_layout[individual_drive_ndx].3 - current_position;
                                let steps_taken = stepper::gpio_stepper_move(
                                    steps_to_take,
                                    hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                                    hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                                    hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                                    true,
                                    hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
                                );
                                *position_vertical.borrow_mut() -= steps_taken.unwrap();
                            } else {
                                // move arm down
                                let steps_to_take =
                                    drive_layout[individual_drive_ndx].3 - current_position;
                                let steps_taken = stepper::gpio_stepper_move(
                                    steps_to_take,
                                    hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                                    hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                                    hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM,
                                    false,
                                    hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
                                );
                                *position_vertical.borrow_mut() += steps_taken.unwrap();
                            }
                            // rabbitmq open drive tray
                            rabbitmq::rabbitmq_publish(
                                rabbit_channel.clone(),
                                drive_layout[individual_drive_ndx].0.to_string().as_str(),
                                "{{'Type': 'eject'}}".to_string(),
                            )
                            .await
                            .unwrap();
                            // allow time to open tray
                            sleep(Duration::from_secs(5)).await;
                            // lower vertitcal to drop position
                            let steps_taken = stepper::gpio_stepper_move(
                                50, // TODO put in real number
                                hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
                                hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
                                hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_ASSEMBLY,
                                false,
                                hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
                            );
                            *position_vertical.borrow_mut() -= steps_taken.unwrap();
                            // drop media into tray
                            let _result =
                                gpio::gpio_set_pin(false, hardware_layout::GPIO_RELAY_VACUUM);
                            // rabbitmq close/start rip
                            let mut ripper_software = "makemkv";
                            if choice_spindle_1_media_type.choice().unwrap().as_str()
                                == hardware_layout::DRIVETYPE_CD
                            {
                                ripper_software = "abcde";
                            }
                            rabbitmq::rabbitmq_publish(
                                rabbit_channel.clone(),
                                drive_layout[individual_drive_ndx].0.to_string().as_str(),
                                format!(
                                    "{{'Type': '{}', 'UUID': '{:?}'}}",
                                    ripper_software, path_photo_name
                                )
                                .to_string(),
                            )
                            .await
                            .unwrap();
                            drive_layout[individual_drive_ndx].4 = true;
                        } else {
                            spindle_one_media_left = false;
                            break;
                        }
                    }
                }
            }
        }
        if spindle_two_media_left {
            // TODO do I zero first or do math?
            let steps_taken = stepper::gpio_stepper_move(
                hardware_layout::INPUT_SPINDLE_LOCATIONS[1],
                hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                true,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
            );
            *position_horizontal.borrow_mut() += steps_taken.unwrap();
        }
        if spindle_three_media_left {
            // TODO do math
            let steps_taken = stepper::gpio_stepper_move(
                hardware_layout::INPUT_SPINDLE_LOCATIONS[2],
                hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                true,
                hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
            );
            *position_horizontal.borrow_mut() += steps_taken.unwrap();
        }
        sleep(Duration::from_secs(1)).await;
    }
    rabbitmq::rabbitmq_close(rabbit_channel, rabbit_connection);
}
