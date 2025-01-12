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
use uuid::Uuid;

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
// Main movement arm
const GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT: u8 = 255;
const GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT: u8 = 255;
const GPIO_STEPPER_HORIZONTAL_DIRECTION: u8 = 26;
const GPIO_STEPPER_HORIZONTAL_PULSE: u8 = 19;

// CD Picker/loader
const GPIO_STEPPER_VERTICAL_END_STOP_ASSEMBLY: u8 = 255;
const GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM: u8 = 255;
const GPIO_STEPPER_VERTICAL_END_STOP_TOP: u8 = 255;
const GPIO_STEPPER_VERTICAL_DIRECTION: u8 = 11;
const GPIO_STEPPER_VERTICAL_PULSE: u8 = 9;

// Image tray
const GPIO_STEPPER_TRAY_END_STOP_BACK: u8 = 255;
const GPIO_STEPPER_TRAY_END_STOP_FRONT: u8 = 255;
const GPIO_STEPPER_TRAY_DIRECTION: u8 = 4;
const GPIO_STEPPER_TRAY_PULSE: u8 = 3;

const GPIO_RELAY_VACUUM: u8 = 255;
const GPIO_RELAY_LIGHT: u8 = 255;

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
        (22, vec![hardware_layout::DRIVETYPE_UHD], 600, 300, false),
        (23, vec![hardware_layout::DRIVETYPE_UHD], 600, 400, false),
        // top row of drives
        (24, vec![hardware_layout::DRIVETYPE_HDDVD], 700, 150, false),
    ];
    // connect to database
    //let db_pool = database::database_open().unwrap();
    let (_rabbit_connection, rabbit_channel) =
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

    let mut win = Window::new(0, 0, 800, 480, "pi_terminal for autoripper");

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
    let mut button_vacuum = Button::new(600, 310, 80, 50, "Vacuum");
    let mut button_snapshot = Button::new(700, 310, 80, 50, "Snapshot");

    // start/stop ripping
    let mut button_start = Button::new(500, 390, 150, 60, "Start Ripping!");
    let mut button_stop = Button::new(650, 390, 150, 60, "Stop!");

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
                    i32::MAX,
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
                    i32::MAX,
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

    app.run().unwrap();

    // launch thread to do the actual processing of the discs
    //    let _handle_tmdb = tokio::spawn(async move {
    //let db_pool = database::database_open().unwrap();
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
            if choice_spindle_1_media_type.choice() != hardware_layout::DRIVETYPE_NONE {
                spindle_one_media_left = true;
            }
            if choice_spindle_2_media_type.choice() != hardware_layout::DRIVETYPE_NONE {
                spindle_two_media_left = true;
            }
            if choice_spindle_3_media_type.choice() != hardware_layout::DRIVETYPE_NONE {
                spindle_three_media_left = true;
            }
            if choice_spindle_4_media_type.choice() != hardware_layout::DRIVETYPE_NONE {
                spindle_four_media_left = true;
            }
            initial_start = false;
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
                sleep(Duration::from_secs(5)).await;
                // TODO place arm for media pickup
                // pick up media
                let _result = gpio::gpio_set_pin(true, GPIO_RELAY_VACUUM);
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
                // drop media
                let _result = gpio::gpio_set_pin(false, GPIO_RELAY_VACUUM);
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
                        .contains(&choice_spindle_1_media_type.choice().as_str())
                    {
                        // raise the vertical to above the spindle
                        let steps_taken = stepper::gpio_stepper_move(
                            hardware_layout::SPINDLE_HEIGHT,
                            GPIO_STEPPER_VERTICAL_PULSE,
                            GPIO_STEPPER_VERTICAL_DIRECTION,
                            GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                            true,
                        );
                        *position_vertical.borrow_mut() += steps_taken.unwrap();
                        // zero out the horizontal
                        let steps_taken = stepper::gpio_stepper_move(
                            i32::MAX,
                            GPIO_STEPPER_HORIZONTAL_PULSE,
                            GPIO_STEPPER_HORIZONTAL_DIRECTION,
                            GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
                            false,
                        );
                        *position_horizontal.borrow_mut() -= steps_taken.unwrap();
                        // move horizontal to spindle one location
                        let steps_taken = stepper::gpio_stepper_move(
                            hardware_layout::INPUT_SPINDLE_LOCATIONS[0],
                            GPIO_STEPPER_HORIZONTAL_PULSE,
                            GPIO_STEPPER_HORIZONTAL_DIRECTION,
                            GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                            true,
                        );
                        *position_horizontal.borrow_mut() += steps_taken.unwrap();
                        // lower vertical to stop switch on assembly
                        let steps_taken = stepper::gpio_stepper_move(
                            5000, // TODO put in real number
                            GPIO_STEPPER_VERTICAL_PULSE,
                            GPIO_STEPPER_VERTICAL_DIRECTION,
                            GPIO_STEPPER_VERTICAL_END_STOP_ASSEMBLY,
                            false,
                        );
                        *position_vertical.borrow_mut() -= steps_taken.unwrap();
                        // TODO if spindle stop switch triggers, turn off media switch and break
                        if *position_vertical.borrow() >= 0 {
                            // pick up media from spindle
                            let _result: Result<(), Box<dyn Error>> =
                                gpio::gpio_set_pin(true, GPIO_RELAY_VACUUM);
                            // raise vertical to the camera level
                            let steps_taken = stepper::gpio_stepper_move(
                                hardware_layout::CAMERA_LOCATION.1,
                                GPIO_STEPPER_VERTICAL_PULSE,
                                GPIO_STEPPER_VERTICAL_DIRECTION,
                                GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                                true,
                            );
                            *position_vertical.borrow_mut() += steps_taken.unwrap();
                            // move horizontal to camera
                            let steps_taken = stepper::gpio_stepper_move(
                                hardware_layout::CAMERA_LOCATION.0,
                                GPIO_STEPPER_HORIZONTAL_PULSE,
                                GPIO_STEPPER_HORIZONTAL_DIRECTION,
                                GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                                true,
                            );
                            *position_horizontal.borrow_mut() += steps_taken.unwrap();
                            // move camera plate out
                            let steps_taken = stepper::gpio_stepper_move(
                                hardware_layout::CAMERA_PLATE_STEPS,
                                GPIO_STEPPER_TRAY_PULSE,
                                GPIO_STEPPER_TRAY_DIRECTION,
                                GPIO_STEPPER_TRAY_END_STOP_FRONT,
                                true,
                            );
                            *position_camera_tray.borrow_mut() += steps_taken.unwrap();
                            // turn on led
                            let _result: Result<(), Box<dyn Error>> =
                                gpio::gpio_set_pin(true, GPIO_RELAY_LIGHT);
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
                                gpio::gpio_set_pin(false, GPIO_RELAY_LIGHT);
                            // move camera plate in
                            let steps_taken = stepper::gpio_stepper_move(
                                hardware_layout::CAMERA_PLATE_STEPS,
                                GPIO_STEPPER_TRAY_PULSE,
                                GPIO_STEPPER_TRAY_DIRECTION,
                                GPIO_STEPPER_TRAY_END_STOP_FRONT,
                                false,
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
                                    GPIO_STEPPER_HORIZONTAL_PULSE,
                                    GPIO_STEPPER_HORIZONTAL_DIRECTION,
                                    GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                                    true,
                                );
                                *position_horizontal.borrow_mut() -= steps_taken.unwrap();
                            } else {
                                // move arm left
                                let steps_to_take =
                                    drive_layout[individual_drive_ndx].2 - current_position;
                                let steps_taken = stepper::gpio_stepper_move(
                                    steps_to_take,
                                    GPIO_STEPPER_HORIZONTAL_PULSE,
                                    GPIO_STEPPER_HORIZONTAL_DIRECTION,
                                    GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
                                    false,
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
                                    GPIO_STEPPER_VERTICAL_PULSE,
                                    GPIO_STEPPER_VERTICAL_DIRECTION,
                                    GPIO_STEPPER_VERTICAL_END_STOP_TOP,
                                    true,
                                );
                                *position_vertical.borrow_mut() -= steps_taken.unwrap();
                            } else {
                                // move arm down
                                let steps_to_take =
                                    drive_layout[individual_drive_ndx].3 - current_position;
                                let steps_taken = stepper::gpio_stepper_move(
                                    steps_to_take,
                                    GPIO_STEPPER_VERTICAL_PULSE,
                                    GPIO_STEPPER_VERTICAL_DIRECTION,
                                    GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM,
                                    false,
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
                                GPIO_STEPPER_VERTICAL_PULSE,
                                GPIO_STEPPER_VERTICAL_DIRECTION,
                                GPIO_STEPPER_VERTICAL_END_STOP_ASSEMBLY,
                                false,
                            );
                            *position_vertical.borrow_mut() -= steps_taken.unwrap();
                            // drop media into tray
                            let _result = gpio::gpio_set_pin(false, GPIO_RELAY_VACUUM);
                            // rabbitmq close/start rip
                            let mut ripper_software = "makemkv";
                            if choice_spindle_1_media_type.choice().as_str()
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
                GPIO_STEPPER_HORIZONTAL_PULSE,
                GPIO_STEPPER_HORIZONTAL_DIRECTION,
                GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                true,
            );
            *position_horizontal.borrow_mut() += steps_taken.unwrap();
        }
        if spindle_three_media_left {
            // TODO do I zero first or do math?
            let steps_taken = stepper::gpio_stepper_move(
                hardware_layout::INPUT_SPINDLE_LOCATIONS[2],
                GPIO_STEPPER_HORIZONTAL_PULSE,
                GPIO_STEPPER_HORIZONTAL_DIRECTION,
                GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                true,
            );
            *position_horizontal.borrow_mut() += steps_taken.unwrap();
        }
        if spindle_four_media_left {
            // TODO do I zero first or do math?
            let steps_taken = stepper::gpio_stepper_move(
                hardware_layout::INPUT_SPINDLE_LOCATIONS[3],
                GPIO_STEPPER_HORIZONTAL_PULSE,
                GPIO_STEPPER_HORIZONTAL_DIRECTION,
                GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT,
                true,
            );
            *position_horizontal.borrow_mut() += steps_taken.unwrap();
        }
        sleep(Duration::from_secs(1)).await;
    }

//    app.run().unwrap();
}
