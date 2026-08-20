fn main() {
    day::launch(
        day::WindowOptions {
            // Tag the window with its native toolkit so the six side-by-side launches are
            // distinguishable: "Day Skies (AppKit)" / "(GTK)" / "(Qt)".
            title: format!("Day Skies ({})", day::toolkit_name()),
            // A desktop-appropriate default size; mobile fills the screen regardless.
            size: day::prelude::Size::new(960.0, 640.0),
            min_size: Some(day::prelude::Size::new(620.0, 480.0)),
            app_name: Some("Day Skies".into()),
            ..Default::default()
        },
        day_skies::root,
    );
}
