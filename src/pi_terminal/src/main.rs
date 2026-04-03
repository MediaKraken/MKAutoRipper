use fltk::{
    app,
    button::Button,
    enums::{Color, FrameType},
    frame::Frame,
    group::{Pack, PackType},
    menu::Choice,
    prelude::*,
    window::Window,
};
use fltk_table::{SmartTable, TableOpts};
use std::{cell::RefCell, error::Error, fs, path::Path, rc::Rc};
use tokio::time::{sleep, Duration};

mod byte_size;
mod camera;
mod database;
mod gpio;
mod hardware_layout;
mod rabbitmq;
mod servo;
mod stepper;

type AppResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MediaType {
    None,
    Cd,
    Dvd,
    Bluray,
    Uhd,
    Hddvd,
}

impl MediaType {
    fn as_str(self) -> &'static str {
        match self {
            MediaType::None => hardware_layout::DRIVETYPE_NONE,
            MediaType::Cd => hardware_layout::DRIVETYPE_CD,
            MediaType::Dvd => hardware_layout::DRIVETYPE_DVD,
            MediaType::Bluray => hardware_layout::DRIVETYPE_BRAY,
            MediaType::Uhd => hardware_layout::DRIVETYPE_UHD,
            MediaType::Hddvd => hardware_layout::DRIVETYPE_HDDVD,
        }
    }

    fn from_choice_index(idx: i32) -> Self {
        match idx {
            1 => MediaType::Cd,
            2 => MediaType::Dvd,
            3 => MediaType::Bluray,
            4 => MediaType::Uhd,
            5 => MediaType::Hddvd,
            _ => MediaType::None,
        }
    }
}

#[derive(Debug, Clone)]
struct DriveSlot {
    id: u16,
    supported_media: Vec<MediaType>,
    x: i32,
    y: i32,
    in_use: bool,
}

#[derive(Debug, Default, Clone, Copy)]
struct Position {
    horizontal: i32,
    vertical: i32,
    tray: i32,
}

#[derive(Debug)]
struct Services {
    db_pool: database::DatabasePoolType,
    rabbit_channel: rabbitmq::RabbitChannelType,
}

#[derive(Debug)]
struct MachineState {
    position: Position,
    vacuum_on: bool,
    hard_stop: bool,
    drives: Vec<DriveSlot>,
}

impl MachineState {
    fn new() -> Self {
        Self {
            position: Position::default(),
            vacuum_on: false,
            hard_stop: false,
            drives: build_drive_layout(),
        }
    }
}

#[derive(Clone)]
struct UiHandles {
    frame_horizontal: Frame,
    frame_vertical: Frame,
    frame_tray: Frame,
    action_choice: Choice,
    spindle_1_choice: Choice,
    spindle_2_choice: Choice,
    spindle_3_choice: Choice,
}

fn build_drive_layout() -> Vec<DriveSlot> {
    vec![
        // CD/DVD rows
        DriveSlot { id: 0, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[0], y: hardware_layout::DRIVE_ROW_LOCATIONS[0], in_use: false },
        DriveSlot { id: 1, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[1], y: hardware_layout::DRIVE_ROW_LOCATIONS[0], in_use: false },
        DriveSlot { id: 2, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[2], y: hardware_layout::DRIVE_ROW_LOCATIONS[0], in_use: false },
        DriveSlot { id: 3, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[3], y: hardware_layout::DRIVE_ROW_LOCATIONS[0], in_use: false },

        DriveSlot { id: 4, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[0], y: hardware_layout::DRIVE_ROW_LOCATIONS[1], in_use: false },
        DriveSlot { id: 5, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[1], y: hardware_layout::DRIVE_ROW_LOCATIONS[1], in_use: false },
        DriveSlot { id: 6, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[2], y: hardware_layout::DRIVE_ROW_LOCATIONS[1], in_use: false },
        DriveSlot { id: 7, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[3], y: hardware_layout::DRIVE_ROW_LOCATIONS[1], in_use: false },

        DriveSlot { id: 8, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[0], y: hardware_layout::DRIVE_ROW_LOCATIONS[2], in_use: false },
        DriveSlot { id: 9, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[1], y: hardware_layout::DRIVE_ROW_LOCATIONS[2], in_use: false },
        DriveSlot { id: 10, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[2], y: hardware_layout::DRIVE_ROW_LOCATIONS[2], in_use: false },
        DriveSlot { id: 11, supported_media: vec![MediaType::Cd, MediaType::Dvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[3], y: hardware_layout::DRIVE_ROW_LOCATIONS[2], in_use: false },

        // Bluray rows
        DriveSlot { id: 12, supported_media: vec![MediaType::Bluray], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[0], y: hardware_layout::DRIVE_ROW_LOCATIONS[3], in_use: false },
        DriveSlot { id: 13, supported_media: vec![MediaType::Bluray], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[1], y: hardware_layout::DRIVE_ROW_LOCATIONS[3], in_use: false },
        DriveSlot { id: 14, supported_media: vec![MediaType::Bluray], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[2], y: hardware_layout::DRIVE_ROW_LOCATIONS[3], in_use: false },
        DriveSlot { id: 15, supported_media: vec![MediaType::Bluray], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[3], y: hardware_layout::DRIVE_ROW_LOCATIONS[3], in_use: false },

        DriveSlot { id: 16, supported_media: vec![MediaType::Bluray], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[0], y: hardware_layout::DRIVE_ROW_LOCATIONS[4], in_use: false },
        DriveSlot { id: 17, supported_media: vec![MediaType::Bluray], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[1], y: hardware_layout::DRIVE_ROW_LOCATIONS[4], in_use: false },
        DriveSlot { id: 18, supported_media: vec![MediaType::Bluray], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[2], y: hardware_layout::DRIVE_ROW_LOCATIONS[4], in_use: false },
        DriveSlot { id: 19, supported_media: vec![MediaType::Bluray], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[3], y: hardware_layout::DRIVE_ROW_LOCATIONS[4], in_use: false },

        // UHD row
        DriveSlot { id: 20, supported_media: vec![MediaType::Uhd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[0], y: hardware_layout::DRIVE_ROW_LOCATIONS[5], in_use: false },
        DriveSlot { id: 21, supported_media: vec![MediaType::Uhd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[1], y: hardware_layout::DRIVE_ROW_LOCATIONS[5], in_use: false },
        DriveSlot { id: 22, supported_media: vec![MediaType::Uhd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[2], y: hardware_layout::DRIVE_ROW_LOCATIONS[5], in_use: false },
        DriveSlot { id: 23, supported_media: vec![MediaType::Uhd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[3], y: hardware_layout::DRIVE_ROW_LOCATIONS[5], in_use: false },

        // HDDVD
        DriveSlot { id: 24, supported_media: vec![MediaType::Hddvd], x: hardware_layout::DRIVE_COLUMN_LOCATIONS[4], y: hardware_layout::DRIVE_ROW_LOCATIONS[6], in_use: false },
    ]
}

fn find_steps_to_take(choice_index: i32) -> i32 {
    match choice_index {
        0 => 1,
        1 => 10,
        2 => 100,
        3 => 500,
        4 => 1_000,
        5 => 5_000,
        6 => 10_000,
        7 => 25_000,
        8 => 50_000,
        9 => 100_000,
        _ => 1,
    }
}

struct MachineController {
    state: Rc<RefCell<MachineState>>,
    ui: UiHandles,
}

impl MachineController {
    fn new(state: Rc<RefCell<MachineState>>, ui: UiHandles) -> Self {
        Self { state, ui }
    }

    fn refresh_position_labels(&mut self) {
        let state = self.state.borrow();
        self.ui
            .frame_horizontal
            .set_label(&format!("Horiz: {}", state.position.horizontal));
        self.ui
            .frame_vertical
            .set_label(&format!("Vert: {}", state.position.vertical));
        self.ui
            .frame_tray
            .set_label(&format!("Tray: {}", state.position.tray));
    }

    fn move_horizontal(&mut self, steps: i32, clockwise: bool) -> AppResult<()> {
        let hard_stop_pin = if clockwise {
            hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT
        } else {
            hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_RIGHT
        };

        let taken = stepper::gpio_stepper_move(
            steps.abs(),
            hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
            hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
            hard_stop_pin,
            clockwise,
            hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
        )?;

        let mut state = self.state.borrow_mut();
        if clockwise {
            state.position.horizontal += taken;
        } else {
            state.position.horizontal -= taken;
        }
        drop(state);
        self.refresh_position_labels();
        Ok(())
    }

    fn move_vertical(&mut self, steps: i32, upward: bool) -> AppResult<()> {
        let hard_stop_pin = if upward {
            hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_TOP
        } else {
            hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_BOTTOM
        };

        let taken = stepper::gpio_stepper_move(
            steps.abs(),
            hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
            hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
            hard_stop_pin,
            upward,
            hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
        )?;

        let mut state = self.state.borrow_mut();
        if upward {
            state.position.vertical += taken;
        } else {
            state.position.vertical -= taken;
        }
        drop(state);
        self.refresh_position_labels();
        Ok(())
    }

    fn move_tray(&mut self, steps: i32, backward: bool) -> AppResult<()> {
        let hard_stop_pin = if backward {
            hardware_layout::GPIO_STEPPER_TRAY_END_STOP_BACK
        } else {
            hardware_layout::GPIO_STEPPER_TRAY_END_STOP_FRONT
        };

        let taken = stepper::gpio_stepper_move(
            steps.abs(),
            hardware_layout::GPIO_STEPPER_TRAY_PULSE,
            hardware_layout::GPIO_STEPPER_TRAY_DIRECTION,
            hard_stop_pin,
            backward,
            hardware_layout::GPIO_STEPPER_TRAY_MOTOR_SPEED,
        )?;

        let mut state = self.state.borrow_mut();
        if backward {
            state.position.tray += taken;
        } else {
            state.position.tray -= taken;
        }
        drop(state);
        self.refresh_position_labels();
        Ok(())
    }

    fn zero_everything(&mut self) -> AppResult<()> {
        // tray
        stepper::gpio_stepper_move(
            i32::MAX,
            hardware_layout::GPIO_STEPPER_TRAY_PULSE,
            hardware_layout::GPIO_STEPPER_TRAY_DIRECTION,
            hardware_layout::GPIO_STEPPER_TRAY_END_STOP_BACK,
            false,
            hardware_layout::GPIO_STEPPER_TRAY_MOTOR_SPEED,
        )?;
        self.state.borrow_mut().position.tray = 0;

        // vertical
        stepper::gpio_stepper_move(
            i32::MAX,
            hardware_layout::GPIO_STEPPER_VERTICAL_PULSE,
            hardware_layout::GPIO_STEPPER_VERTICAL_DIRECTION,
            hardware_layout::GPIO_STEPPER_VERTICAL_END_STOP_TOP,
            true,
            hardware_layout::GPIO_STEPPER_VERTICAL_MOTOR_SPEED,
        )?;
        self.state.borrow_mut().position.vertical = 0;

        // horizontal
        stepper::gpio_stepper_move(
            i32::MAX,
            hardware_layout::GPIO_STEPPER_HORIZONTAL_PULSE,
            hardware_layout::GPIO_STEPPER_HORIZONTAL_DIRECTION,
            hardware_layout::GPIO_STEPPER_HORIZONTAL_END_STOP_LEFT,
            true,
            hardware_layout::GPIO_STEPPER_HORIZONTAL_MOTOR_SPEED,
        )?;
        self.state.borrow_mut().position.horizontal = 0;

        self.refresh_position_labels();
        Ok(())
    }

    fn execute_action_choice(&mut self) -> AppResult<()> {
        // live read at callback time; avoids stale captured coordinates
        let action = self.ui.action_choice.choice().unwrap_or_default();
        let current = self.state.borrow().position;

        match action.as_str() {
            "Horizontal to 0" => {
                self.state.borrow_mut().position.horizontal = 0;
                self.refresh_position_labels();
                Ok(())
            }
            "Vertical to 0" => {
                self.state.borrow_mut().position.vertical = 0;
                self.refresh_position_labels();
                Ok(())
            }
            "Camera to 0" => {
                self.state.borrow_mut().position.tray = 0;
                self.refresh_position_labels();
                Ok(())
            }
            "Input One" => self.move_horizontal_to(hardware_layout::INPUT_SPINDLE_LOCATIONS[0], current.horizontal),
            "Input Two" => self.move_horizontal_to(hardware_layout::INPUT_SPINDLE_LOCATIONS[1], current.horizontal),
            "Input Three" => self.move_horizontal_to(hardware_layout::INPUT_SPINDLE_LOCATIONS[2], current.horizontal),
            "Output One" => self.move_horizontal_to(hardware_layout::OUTPUT_SPINDLE_LOCATIONS[0], current.horizontal),
            "Output Two" => self.move_horizontal_to(hardware_layout::OUTPUT_SPINDLE_LOCATIONS[1], current.horizontal),
            "Output Three" => self.move_horizontal_to(hardware_layout::OUTPUT_SPINDLE_LOCATIONS[2], current.horizontal),
            "Output Four" => self.move_horizontal_to(hardware_layout::OUTPUT_SPINDLE_LOCATIONS[3], current.horizontal),
            "Drive Column One" => self.move_horizontal_to(hardware_layout::DRIVE_COLUMN_LOCATIONS[0], current.horizontal),
            "Drive Column Two" => self.move_horizontal_to(hardware_layout::DRIVE_COLUMN_LOCATIONS[1], current.horizontal),
            "Drive Column Three" => self.move_horizontal_to(hardware_layout::DRIVE_COLUMN_LOCATIONS[2], current.horizontal),
            "Drive Column Four" => self.move_horizontal_to(hardware_layout::DRIVE_COLUMN_LOCATIONS[3], current.horizontal),
            _ => Ok(()),
        }
    }

    fn move_horizontal_to(&mut self, target: i32, current: i32) -> AppResult<()> {
        if target == current {
            return Ok(());
        }
        let delta = (target - current).abs();
        let clockwise = target > current;
        self.move_horizontal(delta, clockwise)
    }
}

fn make_media_choice(x: i32, y: i32) -> Choice {
    let mut choice = Choice::new(x, y, 120, 30, None);
    choice.add_choice(&format!(
        "{}|{}|{}|{}|{}|{}",
        hardware_layout::DRIVETYPE_NONE,
        hardware_layout::DRIVETYPE_CD,
        hardware_layout::DRIVETYPE_DVD,
        hardware_layout::DRIVETYPE_BRAY,
        hardware_layout::DRIVETYPE_UHD,
        hardware_layout::DRIVETYPE_HDDVD
    ));
    choice.set_value(0);
    choice
}

fn build_ui() -> (app::App, Window, UiHandles, Button, Button, Button, Button, Button, Button, Button, Button, Button, Button, Button, Button, Button, Button) {
    let app = app::App::default();
    let mut win = Window::new(0, 0, 800, 500, "pi_terminal for autoripper");

    let mut container_spindle = Pack::new(10, 25, 300, 35, "Spindle Type");
    let spindle_1_choice = make_media_choice(20, 20);
    let spindle_2_choice = make_media_choice(20, 175);
    let spindle_3_choice = make_media_choice(20, 280);
    container_spindle.end();
    container_spindle.set_frame(FrameType::BorderFrame);
    container_spindle.set_color(Color::Black);
    container_spindle.set_type(PackType::Horizontal);

    let mut container_status = Pack::new(10, 90, 320, 375, "Status");
    let _status_table = SmartTable::default()
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
        match fs::read_to_string("/sys/firmware/devicetree/base/model") {
            Ok(pi_model) => info_table.set_cell_value(0, 1, &pi_model),
            Err(_) => info_table.set_cell_value(0, 1, "Unknown"),
        }
    } else {
        info_table.set_cell_value(0, 1, "Desktop");
    }

    info_table.set_cell_value(1, 0, "Memory");
    if let Ok(mem) = sys_info::mem_info() {
        if let Ok(v) = byte_size::mk_lib_common_bytesize(mem.total * 1000) {
            info_table.set_cell_value(1, 1, &v);
        }
    }

    info_table.set_cell_value(2, 0, "Disk");
    if let Ok(disk) = sys_info::disk_info() {
        if let Ok(v) = byte_size::mk_lib_common_bytesize(disk.total * 1000) {
            info_table.set_cell_value(2, 1, &v);
        }
    }

    info_table.set_cell_value(3, 0, "Camera");
    info_table.set_cell_value(3, 1, "N/A");
    info_table.set_cell_value(4, 0, "OS");
    if let Ok(os_release_info) = sys_info::linux_os_release() {
        info_table.set_cell_value(
            4,
            1,
            &format!("{:?} {:?}", os_release_info.name, os_release_info.version_id),
        );
    }
    container_info.end();
    container_info.set_frame(FrameType::BorderFrame);
    container_info.set_color(Color::Black);
    container_info.set_type(PackType::Horizontal);

    let mut container_action = Pack::new(345, 265, 225, 35, "Action Type");
    let mut action_choice = Choice::new(20, 20, 225, 35, None);
    action_choice.add_choice(
        "1 Step|10 Steps|100 Steps|500 Steps|1,000 Steps|5,000 Steps|10,000 Steps|25,000 Steps|50,000 Steps|100,000 Steps|Input One|Input Two|Input Three|Output One|Output Two|Output Three|Output Four|Drive Column One|Drive Column Two|Drive Column Three|Drive Column Four|Column Camera|Column HDDVD|Drive Row One|Drive Row Two|Drive Row Three|Drive Row Four|Row Camera|Row HDDVD|Horizontal to 0|Vertical to 0|Camera to 0"
    );
    action_choice.set_value(0);
    container_action.end();
    container_action.set_frame(FrameType::BorderFrame);
    container_action.set_color(Color::Black);
    container_action.set_type(PackType::Horizontal);

    let button_execute_combobox = Button::new(345, 310, 225, 40, "Execute Combobox");

    let mut container_position = Pack::new(590, 25, 200, 40, "Position - step(s)");
    let frame_horizontal = Frame::default().with_size(40, 20).with_label("Horiz: 0");
    let frame_vertical = Frame::default().with_size(40, 20).with_label("Vert: 0");
    let frame_tray = Frame::default().with_size(40, 20).with_label("Tray: 0");
    container_position.end();
    container_position.set_frame(FrameType::BorderFrame);
    container_position.set_color(Color::Black);
    container_position.set_type(PackType::Vertical);

    let button_zero = Button::new(400, 15, 150, 50, "Zero Everything");
    let button_left = Button::new(590, 100, 50, 50, "L");
    let button_right = Button::new(700, 100, 50, 50, "R");
    let button_up = Button::new(590, 175, 50, 50, "U");
    let button_down = Button::new(700, 175, 50, 50, "D");
    let button_back = Button::new(590, 250, 50, 50, "B");
    let button_forward = Button::new(700, 250, 50, 50, "F");
    let button_vacuum = Button::new(600, 320, 80, 50, "Vacuum");
    let button_snapshot = Button::new(700, 320, 80, 50, "Snapshot");

    win.end();
    win.show();

    let ui = UiHandles {
        frame_horizontal,
        frame_vertical,
        frame_tray,
        action_choice,
        spindle_1_choice,
        spindle_2_choice,
        spindle_3_choice,
    };

    (
        app,
        win,
        ui,
        button_execute_combobox,
        button_zero,
        button_left,
        button_right,
        button_up,
        button_down,
        button_back,
        button_forward,
        button_vacuum,
        button_snapshot,
        Button::default(),
        Button::default(),
        Button::default(),
    )
}

fn register_callbacks(
    controller: Rc<RefCell<MachineController>>,
    button_execute_combobox: &mut Button,
    button_zero: &mut Button,
    button_left: &mut Button,
    button_right: &mut Button,
    button_up: &mut Button,
    button_down: &mut Button,
    button_back: &mut Button,
    button_forward: &mut Button,
    button_vacuum: &mut Button,
    button_snapshot: &mut Button,
) {
    button_execute_combobox.set_callback({
        let controller = controller.clone();
        move |_| {
            if let Err(err) = controller.borrow_mut().execute_action_choice() {
                eprintln!("execute_action_choice failed: {err}");
            }
        }
    });

    button_zero.set_callback({
        let controller = controller.clone();
        move |_| {
            if let Err(err) = controller.borrow_mut().zero_everything() {
                eprintln!("zero_everything failed: {err}");
            }
        }
    });

    button_right.set_callback({
        let controller = controller.clone();
        move |_| {
            let steps = find_steps_to_take(controller.borrow().ui.action_choice.value());
            if let Err(err) = controller.borrow_mut().move_horizontal(steps, true) {
                eprintln!("move right failed: {err}");
            }
        }
    });

    button_left.set_callback({
        let controller = controller.clone();
        move |_| {
            let steps = find_steps_to_take(controller.borrow().ui.action_choice.value());
            if let Err(err) = controller.borrow_mut().move_horizontal(steps, false) {
                eprintln!("move left failed: {err}");
            }
        }
    });

    button_up.set_callback({
        let controller = controller.clone();
        move |_| {
            let steps = find_steps_to_take(controller.borrow().ui.action_choice.value());
            if let Err(err) = controller.borrow_mut().move_vertical(steps, true) {
                eprintln!("move up failed: {err}");
            }
        }
    });

    button_down.set_callback({
        let controller = controller.clone();
        move |_| {
            let steps = find_steps_to_take(controller.borrow().ui.action_choice.value());
            if let Err(err) = controller.borrow_mut().move_vertical(steps, false) {
                eprintln!("move down failed: {err}");
            }
        }
    });

    button_back.set_callback({
        let controller = controller.clone();
        move |_| {
            let steps = find_steps_to_take(controller.borrow().ui.action_choice.value());
            if let Err(err) = controller.borrow_mut().move_tray(steps, true) {
                eprintln!("move tray back failed: {err}");
            }
        }
    });

    button_forward.set_callback({
        let controller = controller.clone();
        move |_| {
            let steps = find_steps_to_take(controller.borrow().ui.action_choice.value());
            if let Err(err) = controller.borrow_mut().move_tray(steps, false) {
                eprintln!("move tray forward failed: {err}");
            }
        }
    });

    button_vacuum.set_callback({
        let controller = controller.clone();
        move |_| {
            let mut c = controller.borrow_mut();
            let new_state = !c.state.borrow().vacuum_on;
            if let Err(err) = gpio::gpio_set_pin(new_state, hardware_layout::GPIO_RELAY_VACUUM) {
                eprintln!("vacuum toggle failed: {err}");
                return;
            }
            c.state.borrow_mut().vacuum_on = new_state;
        }
    });

    button_snapshot.set_callback(move |_| {
        if let Err(err) = gpio::gpio_set_pin(true, hardware_layout::GPIO_RELAY_LIGHT) {
            eprintln!("light on failed: {err}");
            return;
        }

        if let Err(err) = camera::camera_take_image("demo.png") {
            eprintln!("snapshot failed: {err}");
        }

        if let Err(err) = gpio::gpio_set_pin(false, hardware_layout::GPIO_RELAY_LIGHT) {
            eprintln!("light off failed: {err}");
        }
    });
}

async fn init_services() -> AppResult<Services> {
    let db_pool = database::database_open()?;
    let (_rabbit_connection, rabbit_channel) = rabbitmq::rabbitmq_connect("mkterminal").await?;
    Ok(Services {
        db_pool,
        rabbit_channel,
    })
}

#[tokio::main]
async fn main() -> AppResult<()> {
    let _services = init_services().await?;

    let (fltk_app, _win, ui, mut button_execute_combobox, mut button_zero, mut button_left, mut button_right, mut button_up, mut button_down, mut button_back, mut button_forward, mut button_vacuum, mut button_snapshot, _, _, _) = build_ui();

    let state = Rc::new(RefCell::new(MachineState::new()));
    let controller = Rc::new(RefCell::new(MachineController::new(state, ui)));

    register_callbacks(
        controller,
        &mut button_execute_combobox,
        &mut button_zero,
        &mut button_left,
        &mut button_right,
        &mut button_up,
        &mut button_down,
        &mut button_back,
        &mut button_forward,
        &mut button_vacuum,
        &mut button_snapshot,
    );

    fltk_app.run()?;
    Ok(())
}