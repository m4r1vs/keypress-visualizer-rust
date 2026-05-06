mod input;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation};
use gtk4_layer_shell::{Layer, LayerShell};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::time::Duration;

const APP_ID: &str = "io.github.m4r1vs.keypress-visualizer";

#[derive(Deserialize, Debug, Clone)]
struct Config {
    mappings: HashMap<String, String>,
}

fn load_config() -> Config {
    let config_str = fs::read_to_string("default_config.toml").unwrap_or_else(|_| {
        eprintln!("Warning: Could not read default_config.toml, using empty mappings.");
        "[mappings]".to_string()
    });
    toml::from_str(&config_str).unwrap_or_else(|e| {
        eprintln!("Warning: Error parsing config: {}, using empty mappings.", e);
        Config {
            mappings: HashMap::new(),
        }
    })
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(app: &Application) {
    let config = load_config();
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Keypress Visualizer")
        .build();

    // Initialize layer shell
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("keypress-visualizer"));

    // Anchors
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
    window.set_anchor(gtk4_layer_shell::Edge::Left, false);
    window.set_anchor(gtk4_layer_shell::Edge::Right, false);
    window.set_anchor(gtk4_layer_shell::Edge::Top, false);

    // Margin
    window.set_margin(gtk4_layer_shell::Edge::Bottom, 50);

    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(gtk4::Align::Center)
        .build();

    window.set_child(Some(&container));

    // CSS Styling
    let provider = gtk4::CssProvider::new();
    provider.load_from_data("
        label {
            background-color: rgba(30, 30, 30, 0.8);
            color: white;
            padding: 5px 15px;
            border-radius: 8px;
            font-size: 24px;
            font-weight: bold;
            font-family: sans-serif;
            margin: 5px;
        }
        window {
            background-color: transparent;
        }
    ");
    
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    window.present();

    // Communication channel
    let (tx, rx) = std::sync::mpsc::channel::<String>();

    // Find keyboard and start input thread
    if let Some(device_path) = input::find_keyboard_device() {
        println!("Found keyboard: {}", device_path);
        input::spawn_input_thread(device_path, tx);
    } else {
        eprintln!("No keyboard device found matching the pattern!");
    }

    // Poll channel and update UI
    let container_clone = container.clone();
    let mappings = config.mappings;
    glib::timeout_add_local(Duration::from_millis(10), move || {
        while let Ok(key_name) = rx.try_recv() {
            let display_name = mappings.get(&key_name).cloned().unwrap_or(key_name);
            let label = Label::builder().label(&display_name).build();

            container_clone.append(&label);

            let label_clone = label.clone();
            let container_clone_inner = container_clone.clone();
            glib::timeout_add_local_once(Duration::from_secs(2), move || {
                container_clone_inner.remove(&label_clone);
            });
        }
        glib::ControlFlow::Continue
    });
}
