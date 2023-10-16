use fltk::{app, app::*, button::*, enums::*, frame::*, group::*, prelude::*, window::*};
use fltk_table::{SmartTable, TableOpts};
use std::fs;
use std::{cell::RefCell, rc::Rc};
use rppal::gpio::Gpio;
use rppal::i2c::I2c;
use rppal::pwm::{Channel, Pwm};
use rppal::spi::{Bus, Mode, SlaveSelect, Spi};
use rppal::uart::{Parity, Uart};
use std::error::Error;

mod byte_size;
mod camera;
mod choice;

// BCM pin numbering! Do not use physcial pin numbers.
const GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT: u8 = 23;
const GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT: u8 = 23;
const GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM: u8 = 23;
const GPIO_STEPPER_VERTICAL_END_STOP_TOP: u8 = 23;
const GPIO_RELAY_VACUUM: u8 = 23;
const GPIO_RELAY_WATER: u8 = 23;

fn main() {
    let mut uart_horizontal_stepper = Uart::with_path("/dev/ttyAMA0", 115_200, Parity::None, 8, 1);
    let mut uart_vertical_stepper = Uart::with_path("/dev/ttyAMA1",115_200, Parity::None, 8, 1);

    let app = app::App::default();

    let position_horizontal = Rc::new(RefCell::new(0));
    let position_vertical = Rc::new(RefCell::new(0));

    let mut win = Window::default().with_size(800, 480);

    let mut container_spindle = Pack::new(10, 25, 300, 35, "Spindle Type");

    // setup control for spindle media
    let mut choice_spindle_1_media_type = choice::MyChoice::new(20, 20, 80, 30, None);
    choice_spindle_1_media_type.add_choices(&["None", "CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_1_media_type.set_current_choice(0);
    choice_spindle_1_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_1_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_2_media_type = choice::MyChoice::new(20, 120, 80, 30, None);
    choice_spindle_2_media_type.add_choices(&["None", "CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_2_media_type.set_current_choice(0);
    choice_spindle_2_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_2_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_3_media_type = choice::MyChoice::new(20, 220, 80, 30, None);
    choice_spindle_3_media_type.add_choices(&["None", "CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_3_media_type.set_current_choice(0);
    choice_spindle_3_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_3_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_4_media_type = choice::MyChoice::new(20, 320, 80, 30, None);
    choice_spindle_4_media_type.add_choices(&["None", "CD", "DVD", "BRAY", "UHD", "HDDVD"]);
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

    container_position.end();
    container_position.set_frame(FrameType::BorderFrame);
    container_position.set_color(Color::Black);
    container_position.set_type(PackType::Vertical);

    // move the arms around
    let mut button_zero = Button::new(400, 15, 150, 50, "Zero Everything");
    let mut button_left = Button::new(625, 100, 50, 50, "Left");
    let mut button_up = Button::new(675, 75, 50, 50, "Up");
    let mut button_down = Button::new(675, 125, 50, 50, "Down");
    let mut button_right = Button::new(725, 100, 50, 50, "Right");

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
            *position_horizontal.borrow_mut() += 1;
            frame_position_horizontal.set_label(&format!(
                "Horiz: {}",
                &position_horizontal.borrow().to_string()
            ));
        }
        // TODO move arm
    });

    button_left.set_callback(move |_| {
        *position_horizontal.borrow_mut() -= 1;
        frame_position_horizontal.set_label(&format!("Horiz: {}", &position_horizontal.borrow()));
        // TODO move arm
    });

    button_up.set_callback({
        let position_vertical = position_vertical.clone();
        let mut frame_position_vertical = frame_position_vertical.clone();
        move |_| {
            *position_vertical.borrow_mut() += 1;
            frame_position_vertical.set_label(&format!(
                "Vert: {}",
                &position_vertical.borrow().to_string()
            ));
        }
        // TODO move arm
    });

    button_down.set_callback(move |_| {
        *position_vertical.borrow_mut() -= 1;
        frame_position_vertical.set_label(&format!("Vert: {}", &position_vertical.borrow()));
        // TODO move arm
    });

    button_vacuum.set_callback(move |_| {
        // TODO toggle vacuum
    });

    button_snapshot.set_callback(move |_| {
        // TODO take image via raspberry pi camera
    });

    button_water.set_callback(move |_| {
        // TODO toggle water flow
    });

    button_spinner.set_callback(move |_| {
        // TODO toggle media spinner motor
    });

    button_buffer.set_callback(move |_| {
        // TODO toggle cleaner/buffer motor
    });

    button_zero.set_callback(move |_| {
        // TODO move everything to zero
    });

    button_start.set_callback(move |_| {
        // TODO start ripping media
    });

    button_stop.set_callback(move |_| {
        // TODO stop the system immediately
    });

    app.run().unwrap();
}
