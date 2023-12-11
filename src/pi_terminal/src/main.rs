use fltk::{app, app::*, button::*, enums::*, frame::*, group::*, prelude::*, window::*};
use fltk_table::{SmartTable, TableOpts};
use rppal::gpio::Gpio;
use rppal::i2c::I2c;
use rppal::pwm::{Channel, Pwm};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::uart::{Parity, Uart};
use serde_json::{json, Value};
use std::error::Error;
use std::fs;
use std::{cell::RefCell, rc::Rc};
use tokio::time::{sleep, Duration};

mod byte_size;
mod camera;
mod choice;
mod database;
mod gpio;
mod hardware_layout;
mod rabbitmq;
mod servo;
mod stepper;

// BCM pin numbering! Do not use physcial pin numbers.
const GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT: u8 = 12;
const GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT: u8 = 16;
const GPIO_STEPPER_HORIZONTAL_DIRECTION: u8 = 15;
const GPIO_STEPPER_HORIZONTAL_PULSE: u8 = 14;

const GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM: u8 = 26;
const GPIO_STEPPER_VERTICAL_END_STOP_TOP: u8 = 19;
const GPIO_STEPPER_VERTICAL_DIRECTION: u8 = 21;
const GPIO_STEPPER_VERTICAL_PULSE: u8 = 20;

const GPIO_STEPPER_TRAY_END_STOP_BACK: u8 = 255;
const GPIO_STEPPER_TRAY_END_STOP_FRONT: u8 = 255;
const GPIO_STEPPER_TRAY_DIRECTION: u8 = 255;
const GPIO_STEPPER_TRAY_PULSE: u8 = 255;

const GPIO_RELAY_VACUUM: u8 = 22;
const GPIO_RELAY_LIGHT: u8 = 27;
const GPIO_RELAY_WATER: u8 = 17;

#[tokio::main]
async fn main() {
    let mut drive_layout: Vec<(u16, Vec<&str>, u16, u16, bool)> = vec![
        // bottom row of drives
        (
            0,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            100,
            100,
            false,
        ),
        (
            1,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            100,
            200,
            false,
        ),
        (
            2,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            100,
            300,
            false,
        ),
        (
            3,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            100,
            400,
            false,
        ),
        // 2nd row of drives
        (
            4,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            200,
            100,
            false,
        ),
        (
            5,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            200,
            200,
            false,
        ),
        (
            6,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            200,
            300,
            false,
        ),
        (
            7,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            200,
            400,
            false,
        ),
        // 3rd row of drives
        (
            8,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            300,
            100,
            false,
        ),
        (
            9,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            300,
            200,
            false,
        ),
        (
            10,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            300,
            300,
            false,
        ),
        (
            11,
            vec![
                hardware_layout::DRIVETYPE_CD,
                hardware_layout::DRIVETYPE_DVD,
            ],
            300,
            400,
            false,
        ),
        // 4th row of drives
        (12, vec![hardware_layout::DRIVETYPE_BRAY], 400, 100, false),
        (13, vec![hardware_layout::DRIVETYPE_BRAY], 400, 200, false),
        (14, vec![hardware_layout::DRIVETYPE_BRAY], 400, 300, false),
        (15, vec![hardware_layout::DRIVETYPE_BRAY], 400, 400, false),
        // 5th row of drives
        (16, vec![hardware_layout::DRIVETYPE_BRAY], 500, 100, false),
        (17, vec![hardware_layout::DRIVETYPE_BRAY], 500, 200, false),
        (18, vec![hardware_layout::DRIVETYPE_BRAY], 500, 300, false),
        (19, vec![hardware_layout::DRIVETYPE_BRAY], 500, 400, false),
        // 6th row of drives
        (20, vec![hardware_layout::DRIVETYPE_BRAY], 600, 100, false),
        (21, vec![hardware_layout::DRIVETYPE_BRAY], 600, 200, false),
        (22, vec![hardware_layout::DRIVETYPE_BRAY], 600, 300, false),
        (23, vec![hardware_layout::DRIVETYPE_UHD], 600, 400, false),
        // top row of drives
        (24, vec![hardware_layout::DRIVETYPE_HDDVD], 700, 150, false),
        (25, vec![hardware_layout::DRIVETYPE_HDDVD], 700, 300, false),
    ];
    // connect to database
    // let db_pool = database::database_open().unwrap();
    let (_rabbit_connection, rabbit_channel) =
        rabbitmq::rabbitmq_connect("mkaterminal").await.unwrap();
    let mut rabbit_consumer = rabbitmq::rabbitmq_consumer("mkaterminal", &rabbit_channel)
        .await
        .unwrap();
    let mut hard_stop: bool = false;
    let mut gpio_relay_vacuum_on: bool = false;
    let mut gpio_relay_water_on: bool = false;
    let app = app::App::default();

    let position_horizontal = Rc::new(RefCell::new(0));
    let position_vertical = Rc::new(RefCell::new(0));
    let position_camera_tray = Rc::new(RefCell::new(0));

    let mut win = Window::default().with_size(800, 480);

    let mut container_spindle = Pack::new(10, 25, 300, 35, "Spindle Type");

    // setup control for spindle media
    let mut choice_spindle_1_media_type = choice::MyChoice::new(20, 20, 80, 30, None);
    choice_spindle_1_media_type.add_choices(&[
        hardware_layout::DRIVETYPE_NONE,
        hardware_layout::DRIVETYPE_CD,
        hardware_layout::DRIVETYPE_DVD,
        hardware_layout::DRIVETYPE_BRAY,
        hardware_layout::DRIVETYPE_UHD,
        hardware_layout::DRIVETYPE_HDDVD,
    ]);
    choice_spindle_1_media_type.set_current_choice(0);
    choice_spindle_1_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_1_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_2_media_type = choice::MyChoice::new(20, 120, 80, 30, None);
    choice_spindle_2_media_type.add_choices(&[
        hardware_layout::DRIVETYPE_NONE,
        hardware_layout::DRIVETYPE_CD,
        hardware_layout::DRIVETYPE_DVD,
        hardware_layout::DRIVETYPE_BRAY,
        hardware_layout::DRIVETYPE_UHD,
        hardware_layout::DRIVETYPE_HDDVD,
    ]);
    choice_spindle_2_media_type.set_current_choice(0);
    choice_spindle_2_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_2_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_3_media_type = choice::MyChoice::new(20, 220, 80, 30, None);
    choice_spindle_3_media_type.add_choices(&[
        hardware_layout::DRIVETYPE_NONE,
        hardware_layout::DRIVETYPE_CD,
        hardware_layout::DRIVETYPE_DVD,
        hardware_layout::DRIVETYPE_BRAY,
        hardware_layout::DRIVETYPE_UHD,
        hardware_layout::DRIVETYPE_HDDVD,
    ]);
    choice_spindle_3_media_type.set_current_choice(0);
    choice_spindle_3_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_3_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_4_media_type = choice::MyChoice::new(20, 320, 80, 30, None);
    choice_spindle_4_media_type.add_choices(&[
        hardware_layout::DRIVETYPE_NONE,
        hardware_layout::DRIVETYPE_CD,
        hardware_layout::DRIVETYPE_DVD,
        hardware_layout::DRIVETYPE_BRAY,
        hardware_layout::DRIVETYPE_UHD,
        hardware_layout::DRIVETYPE_HDDVD,
    ]);
    choice_spindle_4_media_type.set_current_choice(0);
    choice_spindle_4_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_4_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    container_spindle.end();
    container_spindle.set_frame(FrameType::BorderFrame);
    container_spindle.set_color(Color::Black);
    container_spindle.set_type(PackType::Horizontal);

    let mut container_status = Pack::new(10, 90, 300, 375, "Status");

    let mut status_table = SmartTable::default()
        .with_size(300, 375)
        .center_of_parent()
        .with_opts(TableOpts {
            rows: 6,
            cols: 3,
            editable: false,
            ..Default::default()
        });

    container_status.end();
    container_status.set_frame(FrameType::BorderFrame);
    container_status.set_color(Color::Black);
    container_status.set_type(PackType::Horizontal);

    let mut container_info = Pack::new(325, 90, 250, 250, "Info");

    let mut info_table = SmartTable::default()
        .with_size(250, 350)
        .center_of_parent()
        .with_opts(TableOpts {
            rows: 12,
            cols: 2,
            editable: false,
            ..Default::default()
        });
    info_table.set_cell_value(0, 0, "Model");
    let pi_model = fs::read_to_string("/sys/firmware/devicetree/base/model").unwrap();
    info_table.set_cell_value(0, 1, &pi_model);
    info_table.set_cell_value(1, 0, "Memory");
    info_table.set_cell_value(
        1,
        1,
        &byte_size::mk_lib_common_bytesize(sys_info::mem_info().unwrap().total).unwrap(),
    );
    info_table.set_cell_value(2, 0, "Disk");
    info_table.set_cell_value(
        2,
        1,
        &byte_size::mk_lib_common_bytesize(sys_info::disk_info().unwrap().total).unwrap(),
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

    let mut container_position = Pack::new(625, 25, 150, 40, "Position - step(s)");

    let mut frame_position_horizontal = Frame::default().with_size(40, 20).with_label("Horiz: 0");
    let mut frame_position_vertical = Frame::default().with_size(40, 20).with_label("Vert: 0");
    let mut frame_position_camera_tray = Frame::default().with_size(40, 20).with_label("Tray: 0");

    container_position.end();
    container_position.set_frame(FrameType::BorderFrame);
    container_position.set_color(Color::Black);
    container_position.set_type(PackType::Vertical);

    // move the arms around
    let mut button_zero = Button::new(400, 15, 150, 50, "Zero Everything");
    let mut button_left = Button::new(625, 100, 25, 50, "L");
    let mut button_left_full_rotation = Button::new(650, 100, 25, 50, "L F");
    let mut button_up = Button::new(675, 75, 25, 50, "U");
    let mut button_up_full_rotation = Button::new(700, 75, 25, 50, "U F");
    let mut button_down = Button::new(675, 125, 25, 50, "D");
    let mut button_down_full_rotation = Button::new(700, 125, 25, 50, "D F");
    let mut button_right = Button::new(725, 100, 25, 50, "R");
    let mut button_right_full_rotation = Button::new(750, 100, 25, 50, "R F");
    let mut button_back = Button::new(725, 150, 25, 50, "B");
    let mut button_back_full_rotation = Button::new(750, 150, 25, 50, "B F");
    let mut button_forward = Button::new(725, 150, 25, 50, "F");
    let mut button_forward_full_rotation = Button::new(750, 150, 25, 50, "F F");

    // activate equipment
    let mut button_vacuum = Button::new(620, 180, 80, 50, "Vacuum");
    let mut button_snapshot = Button::new(700, 180, 80, 50, "Snapshot");

    // following are for buffer/cleaner
    let mut button_water = Button::new(585, 240, 70, 50, "Water");
    let mut button_spinner = Button::new(655, 240, 70, 50, "Spinner");
    let mut button_buffer = Button::new(725, 240, 70, 50, "Buffer");

    // start/stop ripping
    let mut button_start = Button::new(500, 420, 150, 60, "Start Ripping!");
    let mut button_stop = Button::new(650, 420, 150, 60, "Stop!");

    win.end();
    win.show();

    // position button click processing
    button_right.set_callback({
        let position_horizontal = position_horizontal.clone();
        let mut frame_position_horizontal = frame_position_horizontal.clone();
        move |_| {
            let steps_taken = stepper::gpio_stepper_move(
                1,
                GPIO_STEPPER_HORIZONTAL_PULSE,
                GPIO_STEPPER_HORIZONTAL_DIRECTION,
                GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                true,
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
                200,
                GPIO_STEPPER_HORIZONTAL_PULSE,
                GPIO_STEPPER_HORIZONTAL_DIRECTION,
                GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                true,
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
        move |_| {
            let steps_taken = stepper::gpio_stepper_move(
                1,
                GPIO_STEPPER_HORIZONTAL_PULSE,
                GPIO_STEPPER_HORIZONTAL_DIRECTION,
                GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
                false,
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
                200,
                GPIO_STEPPER_HORIZONTAL_PULSE,
                GPIO_STEPPER_HORIZONTAL_DIRECTION,
                GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
                false,
            );
            *position_horizontal.borrow_mut() -= steps_taken.unwrap();
            frame_position_horizontal
                .set_label(&format!("Horiz: {}", &position_horizontal.borrow()));
        }
    });

    button_up.set_callback({
        let position_vertical = position_vertical.clone();
        let mut frame_position_vertical = frame_position_vertical.clone();
        move |_| {
            let steps_taken = stepper::gpio_stepper_move(
                1,
                GPIO_STEPPER_VERTICAL_PULSE,
                GPIO_STEPPER_VERTICAL_DIRECTION,
                GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                true,
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
                200,
                GPIO_STEPPER_VERTICAL_PULSE,
                GPIO_STEPPER_VERTICAL_DIRECTION,
                GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                true,
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
        move |_| {
            let steps_taken = stepper::gpio_stepper_move(
                1,
                GPIO_STEPPER_VERTICAL_PULSE,
                GPIO_STEPPER_VERTICAL_DIRECTION,
                GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM,
                false,
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
                200,
                GPIO_STEPPER_VERTICAL_PULSE,
                GPIO_STEPPER_VERTICAL_DIRECTION,
                GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM,
                false,
            );
            *position_vertical.borrow_mut() -= steps_taken.unwrap();
            frame_position_vertical.set_label(&format!("Vert: {}", &position_vertical.borrow()));
        }
    });

    button_vacuum.set_callback(move |_| {
        // toggle vacuum
        gpio_relay_vacuum_on = !gpio_relay_vacuum_on;
        let _result = gpio::gpio_set_pin(gpio_relay_vacuum_on, GPIO_RELAY_VACUUM);
        // let _result = database::database_insert_logs(
        //     &db_pool,
        //     database::LogType::LOG_RELAY_VACCUUM,
        //     &format!("{}", gpio_relay_vacuum_on),
        // );
    });

    button_snapshot.set_callback(move |_| {
        let _result = camera::camera_take_image("demo.png");
        // let _result =
        //     database::database_insert_logs(&db_pool, database::LogType::LOG_SNAPSHOT, "Snapshot");
        // let _result = database::database_update_totals(&db_pool, "images_taken", 1);
    });

    button_water.set_callback(move |_| {
        // toggle water flow
        gpio_relay_water_on = !gpio_relay_water_on;
        // let _result = gpio::gpio_set_pin(gpio_relay_water_on, GPIO_RELAY_WATER);
        // let _result = database::database_insert_logs(
        //     &db_pool,
        //     database::LogType::LOG_RELAY_WATER,
        //     &format!("{}", gpio_relay_vacuum_on),
        // );
    });

    button_spinner.set_callback(move |_| {
        // TODO toggle media spinner motor
    });

    button_buffer.set_callback(move |_| {
        // TODO toggle cleaner/buffer motor
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
                    999999999,
                    GPIO_STEPPER_TRAY_PULSE,
                    GPIO_STEPPER_TRAY_DIRECTION,
                    GPIO_STEPPER_TRAY_END_STOP_BACK,
                    false,
                );
                *position_camera_tray.borrow_mut() -= steps_taken.unwrap();
                frame_position_camera_tray
                    .set_label(&format!("Tray: {}", &position_camera_tray.borrow()));
                // move loader to top
                let steps_taken = stepper::gpio_stepper_move(
                    999999999,
                    GPIO_STEPPER_VERTICAL_PULSE,
                    GPIO_STEPPER_VERTICAL_DIRECTION,
                    GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                    true,
                );
                *position_vertical.borrow_mut() += steps_taken.unwrap();
                frame_position_vertical
                    .set_label(&format!("Vert: {}", &position_vertical.borrow()));
                // move gantry to far left
                let steps_taken = stepper::gpio_stepper_move(
                    999999999,
                    GPIO_STEPPER_HORIZONTAL_PULSE,
                    GPIO_STEPPER_HORIZONTAL_DIRECTION,
                    GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
                    false,
                );
                *position_horizontal.borrow_mut() -= steps_taken.unwrap();
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

    // launch thread to do the actual processing of the discs
    //    let _handle_tmdb = tokio::spawn(async move {
    let db_pool = database::database_open().unwrap();
    let mut initial_start = true;
    let mut spindle_one_media_left = false;
    let mut spindle_two_media_left = false;
    let mut spindle_three_media_left = false;
    let mut spindle_four_media_left = false;
    loop {
        // check for HARD stop
        if hard_stop {
            break;
        }
        if initial_start {
            if choice_spindle_1_media_type.choice() != "None" {
                spindle_one_media_left = true;
            }
            if choice_spindle_2_media_type.choice() != "None" {
                spindle_two_media_left = true;
            }
            if choice_spindle_3_media_type.choice() != "None" {
                spindle_three_media_left = true;
            }
            if choice_spindle_4_media_type.choice() != "None" {
                spindle_four_media_left = true;
            }
            initial_start = false;
        }
        // grab message from rabbitmq if one is available
        let msg = rabbit_consumer.recv().await;
        if let payload = msg.unwrap().content {
            let json_message: Value =
                serde_json::from_str(&String::from_utf8_lossy(&payload.unwrap())).unwrap();
            if json_message["Type"] == "Done" {
                // TODO position horizontal, position vertical
                // send eject rabbitmq message
                rabbitmq::rabbitmq_publish(
                    rabbit_channel.clone(),
                    json_message["Drive_Num"].as_str().unwrap(),
                    "Eject".to_string(),
                )
                .await
                .unwrap();
                // allow time to umount/eject
                sleep(Duration::from_secs(5)).await;
                // TODO place arm for media pickup
                // pick up media
                let _result = gpio::gpio_set_pin(true, GPIO_RELAY_VACUUM);
                // TODO place media at exit spindle
                // drop media
                let _result = gpio::gpio_set_pin(false, GPIO_RELAY_VACUUM);
            }
        }
        // process next disc
        if spindle_one_media_left {}
        if spindle_two_media_left {}
        if spindle_three_media_left {}
        if spindle_four_media_left {}
        sleep(Duration::from_secs(1)).await;
    }
    //    });

    app.run().unwrap();
}
