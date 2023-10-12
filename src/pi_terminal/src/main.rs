use fltk::{app, app::*, button::*, enums::*, frame::*, group::*, prelude::*, window::*};
mod choice;

fn main() {
    let app = app::App::default();
    let mut win = Window::default().with_size(800, 480);

    let mut container_spindle = Pack::new(20, 10, 300, 35, "Spindle Type");

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

    let mut container_status = Pack::new(10, 210, 250, 350, "Status");

    container_status.end();
    container_status.set_frame(FrameType::BorderFrame);
    container_status.set_color(Color::Black);
    container_status.set_type(PackType::Horizontal);

    let mut button_rip = Button::new(600, 210, 120, 40, "Start Ripping!");

    let mut button_stop = Button::new(600, 310, 120, 40, "Stop!");

    win.end();
    win.show();
    app.run().unwrap();
}
