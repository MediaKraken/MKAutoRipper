use fltk::{enums::*, prelude::*, *};
mod choice;

fn main() {
    let app = app::App::default();
    let mut win = window::Window::default().with_size(800, 480);

    // setup control for spindle media
    let mut choice_spindle_1_media_type = choice::MyChoice::new(20, 20, 100, 30, None);
    choice_spindle_1_media_type.add_choices(&["CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_1_media_type.set_current_choice(1);
    choice_spindle_1_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_1_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_2_media_type = choice::MyChoice::new(20, 120, 80, 30, None);
    choice_spindle_2_media_type.add_choices(&["CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_2_media_type.set_current_choice(1);
    choice_spindle_2_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_2_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_3_media_type = choice::MyChoice::new(20, 220, 80, 30, None);
    choice_spindle_3_media_type.add_choices(&["CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_3_media_type.set_current_choice(1);
    choice_spindle_3_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_3_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    // setup control for spindle media
    let mut choice_spindle_4_media_type = choice::MyChoice::new(20, 320, 80, 30, None);
    choice_spindle_4_media_type.add_choices(&["CD", "DVD", "BRAY", "UHD", "HDDVD"]);
    choice_spindle_4_media_type.set_current_choice(1);
    choice_spindle_4_media_type
        .button()
        .set_frame(FrameType::BorderBox);
    choice_spindle_4_media_type
        .frame()
        .set_frame(FrameType::BorderBox);

    win.end();
    win.show();
    app.run().unwrap();
}
