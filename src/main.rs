mod input;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation};
use gtk4_layer_shell::{Layer, LayerShell};
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;
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
    let (tx, rx) = std::sync::mpsc::channel::<(String, i32)>();

    // Find keyboard and start input thread
    if let Some(device_path) = input::find_keyboard_device() {
        println!("Found keyboard: {}", device_path);
        input::spawn_input_thread(device_path, tx);
    } else {
        eprintln!("No keyboard device found matching the pattern!");
    }

    // State for word detection
    struct WordState {
        label: Option<Label>,
        text: String,
        timeout_id: Option<glib::SourceId>,
        pending_space: bool,
    }

    let state = Rc::new(RefCell::new(WordState {
        label: None,
        text: String::new(),
        timeout_id: None,
        pending_space: false,
    }));

    // Poll channel and update UI
    let container_clone = container.clone();
    let mappings = config.mappings;
    let mut modifiers = std::collections::HashSet::<String>::new();

    glib::timeout_add_local(Duration::from_millis(10), move || {
        while let Ok((key_name, value)) = rx.try_recv() {
            let is_modifier = matches!(
                key_name.as_str(),
                "LEFTCTRL" | "RIGHTCTRL" | "LEFTSHIFT" | "RIGHTSHIFT" | "LEFTALT" | "RIGHTALT" | "LEFTMETA" | "RIGHTMETA"
            );

            if value == 1 { // Press
                if is_modifier {
                    modifiers.insert(key_name.clone());
                    
                    // Non-shift modifiers separate words
                    if key_name != "LEFTSHIFT" && key_name != "RIGHTSHIFT" {
                        let mut s = state.borrow_mut();
                        if s.pending_space {
                            let spc_name = mappings.get("SPACE").cloned().unwrap_or("SPC".to_string());
                            let spc_label = Label::builder().label(&spc_name).build();
                            container_clone.append(&spc_label);
                            let sc = spc_label.clone();
                            let cc = container_clone.clone();
                            glib::timeout_add_local_once(Duration::from_secs(2), move || {
                                cc.remove(&sc);
                            });
                        }
                        s.label = None;
                        s.text.clear();
                        s.timeout_id = None;
                        s.pending_space = false;
                    }
                } else {
                    let has_shift = modifiers.contains("LEFTSHIFT") || modifiers.contains("RIGHTSHIFT");
                    let mut active_mods = Vec::new();
                    for m in &["CTRL", "ALT", "META"] {
                        if modifiers.contains(&format!("LEFT{}", m)) || modifiers.contains(&format!("RIGHT{}", m)) {
                            let mod_name = if modifiers.contains(&format!("LEFT{}", m)) {
                                format!("LEFT{}", m)
                            } else {
                                format!("RIGHT{}", m)
                            };
                            active_mods.push(mappings.get(&mod_name).cloned().unwrap_or(mod_name));
                        }
                    }

                    let base_key = mappings.get(&key_name).cloned().unwrap_or(key_name.clone());
                    let is_word_key = active_mods.is_empty() && key_name != "SPACE";

                    if is_word_key {
                        let letter = if has_shift { base_key.to_uppercase() } else { base_key.to_lowercase() };
                        let mut s = state.borrow_mut();
                        
                        if let Some(label) = s.label.clone() {
                            if s.pending_space {
                                s.text.push_str(" ");
                                s.pending_space = false;
                            }
                            s.text.push_str(&letter);
                            label.set_label(&s.text);
                            if let Some(id) = s.timeout_id.take() {
                                id.remove();
                            }
                        } else {
                            s.text = letter;
                            let label = Label::builder().label(&s.text).build();
                            container_clone.append(&label);
                            s.label = Some(label);
                            s.pending_space = false;
                        }

                        let state_for_timer = state.clone();
                        let label_clone = s.label.as_ref().unwrap().clone();
                        let container_clone_inner = container_clone.clone();
                        let id = glib::timeout_add_local_once(Duration::from_secs(2), move || {
                            container_clone_inner.remove(&label_clone);
                            let mut s = state_for_timer.borrow_mut();
                            if s.label.as_ref() == Some(&label_clone) {
                                s.label = None;
                                s.text.clear();
                                s.timeout_id = None;
                                s.pending_space = false;
                            }
                        });
                        s.timeout_id = Some(id);
                    } else if key_name == "SPACE" {
                        let mut s = state.borrow_mut();
                        if s.label.is_some() {
                            if s.pending_space {
                                // Second space, emit the first one explicitly
                                let spc_name = mappings.get("SPACE").cloned().unwrap_or("SPC".to_string());
                                let spc_label = Label::builder().label(&spc_name).build();
                                container_clone.append(&spc_label);
                                let sc = spc_label.clone();
                                let cc = container_clone.clone();
                                glib::timeout_add_local_once(Duration::from_secs(2), move || {
                                    cc.remove(&sc);
                                });
                            }
                            s.pending_space = true;
                            // Reset word timeout
                            if let Some(id) = s.timeout_id.take() {
                                id.remove();
                            }
                            let state_for_timer = state.clone();
                            let label_clone = s.label.as_ref().unwrap().clone();
                            let container_clone_inner = container_clone.clone();
                            let id = glib::timeout_add_local_once(Duration::from_secs(2), move || {
                                container_clone_inner.remove(&label_clone);
                                let mut s = state_for_timer.borrow_mut();
                                if s.label.as_ref() == Some(&label_clone) {
                                    s.label = None;
                                    s.text.clear();
                                    s.timeout_id = None;
                                    s.pending_space = false;
                                }
                            });
                            s.timeout_id = Some(id);
                        } else {
                            // No word, just show SPC
                            let spc_name = mappings.get("SPACE").cloned().unwrap_or("SPC".to_string());
                            let spc_label = Label::builder().label(&spc_name).build();
                            container_clone.append(&spc_label);
                            let sc = spc_label.clone();
                            let cc = container_clone.clone();
                            glib::timeout_add_local_once(Duration::from_secs(2), move || {
                                cc.remove(&sc);
                            });
                        }
                    } else {
                        // Chord
                        {
                            let mut s = state.borrow_mut();
                            if s.pending_space {
                                let spc_name = mappings.get("SPACE").cloned().unwrap_or("SPC".to_string());
                                let spc_label = Label::builder().label(&spc_name).build();
                                container_clone.append(&spc_label);
                                let sc = spc_label.clone();
                                let cc = container_clone.clone();
                                glib::timeout_add_local_once(Duration::from_secs(2), move || {
                                    cc.remove(&sc);
                                });
                            }
                            s.label = None;
                            s.text.clear();
                            s.timeout_id = None;
                            s.pending_space = false;
                        }

                        let mut chord = active_mods;
                        if has_shift {
                            let shift_name = if modifiers.contains("LEFTSHIFT") { "LEFTSHIFT" } else { "RIGHTSHIFT" };
                            chord.push(mappings.get(shift_name).cloned().unwrap_or("SHIFT".to_string()));
                        }
                        chord.push(base_key.to_uppercase());
                        let display_name = chord.join(" + ");

                        let label = Label::builder().label(&display_name).build();
                        container_clone.append(&label);

                        let label_clone = label.clone();
                        let container_clone_inner = container_clone.clone();
                        glib::timeout_add_local_once(Duration::from_secs(2), move || {
                            container_clone_inner.remove(&label_clone);
                        });
                    }
                }
            } else if value == 0 { // Release
                if is_modifier {
                    modifiers.remove(&key_name);
                }
            }
        }
        glib::ControlFlow::Continue
    });
}
