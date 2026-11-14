//! Generate the shared Fluent resources, native widget catalog, and Swift bridge.
fn main() {
    widget_resources::generate();
    day_build::prebuild_project().expect("day-build: prebuild");
    day_build::bridge::generate().expect("day-build: widget bridge");
}

#[path = "scripts/widget_resources.rs"]
mod widget_resources;
