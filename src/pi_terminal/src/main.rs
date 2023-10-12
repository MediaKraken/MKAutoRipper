use fltk::{app, app::*, button::*, enums::*, frame::*, group::*, prelude::*, window::*};
mod choice;

fn main() {
    let app = app::App::default();
    let mut win = Window::default().with_size(800, 480);

    let mut container_spindle = Pack::new(150, 150, 200, 100, "CounterApplet ");

    // setup control for spindle media
    let mut choice_spindle_1_media_type = choice::MyChoice::new(20, 20, 80, 30, None);
    choice_spindle_1_media_type.add_choices(&["None", "CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_1_media_type.set_current_choice(1);
    choice_spindle_1_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_1_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_2_media_type = choice::MyChoice::new(20, 120, 80, 30, None);
    choice_spindle_2_media_type.add_choices(&["None", "CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_2_media_type.set_current_choice(1);
    choice_spindle_2_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_2_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_3_media_type = choice::MyChoice::new(20, 220, 80, 30, None);
    choice_spindle_3_media_type.add_choices(&["None", "CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_3_media_type.set_current_choice(1);
    choice_spindle_3_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_3_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_4_media_type = choice::MyChoice::new(20, 320, 80, 30, None);
    choice_spindle_4_media_type.add_choices(&["None", "CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_4_media_type.set_current_choice(1);
    choice_spindle_4_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_4_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    container_spindle.end();
    container_spindle.set_frame(FrameType::BorderFrame);
    container_spindle.set_color(Color::Red);
    container_spindle.set_type(PackType::Horizontal);

    let mut button_rip = Button::new(160, 210, 80, 40, "Start Ripping!");

    win.end();
    win.show();
    app.run().unwrap();
}
