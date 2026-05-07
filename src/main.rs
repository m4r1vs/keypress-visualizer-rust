mod config;
mod input;
mod tray;
mod ui;
mod utils;

use gtk4::Application;
use gtk4::prelude::*;

const APP_ID: &str = "io.github.m4r1vs.keypress-visualizer";

fn main() {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(ui::build_ui);
    app.run();
}
