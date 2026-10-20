fn main() {
    day::launch(
        day::WindowOptions::default()
            // Tag the window with its native toolkit so the six side-by-side launches are
            // distinguishable: "Day Skies (AppKit)" / "(GTK)" / "(Qt)".
            .title(format!("Day Skies ({})", day::toolkit_name()))
            // A desktop-appropriate default size; mobile fills the screen regardless.
            .size(960.0, 640.0)
            .min_size(620.0, 480.0)
            .app_name("Day Skies")
            // About reads the version from here on desktop; the mobile entry points stamp it
            // through `day_start!`.
            .version(env!("CARGO_PKG_VERSION")),
        dayapp::root,
    );
}
