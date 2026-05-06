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

fn default_font_size() -> u32 { 24 }
fn default_anchor() -> String { "bottom".to_string() }
fn default_margin_x() -> i32 { 0 }
fn default_margin_y() -> i32 { 50 }
fn default_pos_x_pct() -> f64 { 0.0 }
fn default_pos_y_pct() -> f64 { 0.0 }
fn default_max_keys() -> usize { 10 }
fn default_custom_css() -> String { String::new() }
fn default_spam_threshold() -> usize { 4 }
fn default_spam_hold_ms() -> u64 { 500 }

#[derive(Deserialize, Debug, Clone)]
struct AppearanceConfig {
    #[serde(default = "default_font_size")]
    font_size: u32,
    #[serde(default = "default_anchor")]
    anchor: String,
    #[serde(default = "default_margin_x")]
    margin_x: i32,
    #[serde(default = "default_margin_y")]
    margin_y: i32,
    #[serde(default = "default_pos_x_pct")]
    pos_x_pct: f64,
    #[serde(default = "default_pos_y_pct")]
    pos_y_pct: f64,
    #[serde(default = "default_max_keys")]
    max_keys: usize,
    #[serde(default = "default_custom_css")]
    custom_css: String,
    #[serde(default = "default_spam_threshold")]
    spam_threshold: usize,
    #[serde(default = "default_spam_hold_ms")]
    spam_hold_ms: u64,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            font_size: default_font_size(),
            anchor: default_anchor(),
            margin_x: default_margin_x(),
            margin_y: default_margin_y(),
            pos_x_pct: default_pos_x_pct(),
            pos_y_pct: default_pos_y_pct(),
            max_keys: default_max_keys(),
            custom_css: default_custom_css(),
            spam_threshold: default_spam_threshold(),
            spam_hold_ms: default_spam_hold_ms(),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
struct Config {
    #[serde(default)]
    mappings: HashMap<String, String>,
    #[serde(default)]
    appearance: AppearanceConfig,
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
            appearance: AppearanceConfig::default(),
        }
    })
}

fn main() {
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run();
}

fn map_char_key(key: &str, has_shift: bool) -> Option<&'static str> {
    match (key, has_shift) {
        ("1", true) => Some("!"),
        ("2", true) => Some("@"),
        ("3", true) => Some("#"),
        ("4", true) => Some("$"),
        ("5", true) => Some("%"),
        ("6", true) => Some("^"),
        ("7", true) => Some("&"),
        ("8", true) => Some("*"),
        ("9", true) => Some("("),
        ("0", true) => Some(")"),
        ("MINUS", false) => Some("-"),
        ("MINUS", true) => Some("_"),
        ("EQUAL", false) => Some("="),
        ("EQUAL", true) => Some("+"),
        ("LEFTBRACE", false) => Some("["),
        ("LEFTBRACE", true) => Some("{"),
        ("RIGHTBRACE", false) => Some("]"),
        ("RIGHTBRACE", true) => Some("}"),
        ("BACKSLASH", false) => Some("\\"),
        ("BACKSLASH", true) => Some("|"),
        ("SEMICOLON", false) => Some(";"),
        ("SEMICOLON", true) => Some(":"),
        ("APOSTROPHE", false) => Some("'"),
        ("APOSTROPHE", true) => Some("\""),
        ("GRAVE", false) => Some("`"),
        ("GRAVE", true) => Some("~"),
        ("COMMA", false) => Some(","),
        ("COMMA", true) => Some("<"),
        ("DOT", false) => Some("."),
        ("DOT", true) => Some(">"),
        ("SLASH", false) => Some("/"),
        ("SLASH", true) => Some("?"),
        _ => None,
    }
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

    let anchor_str = config.appearance.anchor.to_lowercase();
    let top = anchor_str.contains("top");
    let bottom = anchor_str.contains("bottom");
    let left = anchor_str.contains("left");
    let right = anchor_str.contains("right");

    window.set_anchor(gtk4_layer_shell::Edge::Top, top);
    window.set_anchor(gtk4_layer_shell::Edge::Bottom, bottom);
    window.set_anchor(gtk4_layer_shell::Edge::Left, left);
    window.set_anchor(gtk4_layer_shell::Edge::Right, right);

    let mut width = 1920.0;
    let mut height = 1080.0;
    if let Some(display) = gtk4::gdk::Display::default() {
        let monitors = display.monitors();
        if let Some(monitor) = monitors.item(0).and_downcast::<gtk4::gdk::Monitor>() {
            let geometry = monitor.geometry();
            width = geometry.width() as f64;
            height = geometry.height() as f64;
        }
    }

    let mut m_top = config.appearance.margin_y;
    let m_bottom = config.appearance.margin_y;
    let mut m_left = config.appearance.margin_x;
    let m_right = config.appearance.margin_x;

    if config.appearance.pos_x_pct > 0.0 {
        let px_x = (width * config.appearance.pos_x_pct / 100.0) as i32;
        m_left += px_x;
    }
    if config.appearance.pos_y_pct > 0.0 {
        let px_y = (height * config.appearance.pos_y_pct / 100.0) as i32;
        m_top += px_y;
    }

    if top { window.set_margin(gtk4_layer_shell::Edge::Top, m_top); }
    if bottom { window.set_margin(gtk4_layer_shell::Edge::Bottom, m_bottom); }
    if left { window.set_margin(gtk4_layer_shell::Edge::Left, m_left); }
    if right { window.set_margin(gtk4_layer_shell::Edge::Right, m_right); }

    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(gtk4::Align::Center)
        .build();

    window.set_child(Some(&container));

    // CSS Styling
    let provider = gtk4::CssProvider::new();
    let css = if config.appearance.custom_css.trim().is_empty() {
        format!("\
            label {{
                background-color: rgba(30, 30, 30, 0.8);
                color: white;
                padding: 5px 15px;
                border-radius: 8px;
                font-size: {}px;
                font-weight: bold;
                font-family: sans-serif;
                margin: 5px;
            }}
            window {{
                background-color: transparent;
            }}
        ", config.appearance.font_size)
    } else {
        config.appearance.custom_css.clone()
    };
    provider.load_from_data(&css);
    
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
    }

    struct SpamState {
        key_name: String,
        display_name: String,
        count: usize,
        label: Option<Label>,
        first_press_time: std::time::Instant,
        timeout_id: Option<glib::SourceId>,
        repeat_ticks: usize,
    }

    let state = Rc::new(RefCell::new(WordState {
        label: None,
        text: String::new(),
        timeout_id: None,
    }));

    let spam_state: Rc<RefCell<Option<SpamState>>> = Rc::new(RefCell::new(None));

    // Poll channel and update UI
    let container_clone = container.clone();
    let mappings = config.mappings;
    let mut modifiers = std::collections::HashSet::<String>::new();
    let spam_threshold = config.appearance.spam_threshold;
    let spam_hold_ms = config.appearance.spam_hold_ms as u128;

    glib::timeout_add_local(Duration::from_millis(10), move || {
        while let Ok((key_name, value)) = rx.try_recv() {
            let is_modifier = matches!(
                key_name.as_str(),
                "LEFTCTRL" | "RIGHTCTRL" | "LEFTSHIFT" | "RIGHTSHIFT" | "LEFTALT" | "RIGHTALT" | "LEFTMETA" | "RIGHTMETA"
            );

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
            let mut mapped_char = None;
            if !mappings.contains_key(&key_name) {
                if let Some(mapped) = map_char_key(&key_name, has_shift) {
                    mapped_char = Some(mapped.to_string());
                }
            }
            let is_char = key_name.len() == 1 || mapped_char.is_some();
            let is_word_key = active_mods.is_empty() && is_char;

            let display_name = if is_word_key {
                if let Some(mc) = mapped_char.clone() {
                    mc.to_string()
                } else {
                    if has_shift { base_key.to_uppercase() } else { base_key.to_lowercase() }
                }
            } else if key_name == "SPACE" {
                mappings.get("SPACE").cloned().unwrap_or("SPC".to_string())
            } else if key_name == "BACKSPACE" {
                mappings.get("BACKSPACE").cloned().unwrap_or("⌫".to_string())
            } else {
                let mut chord = active_mods.clone();
                if has_shift {
                    let shift_name = if modifiers.contains("LEFTSHIFT") { "LEFTSHIFT" } else { "RIGHTSHIFT" };
                    chord.push(mappings.get(shift_name).cloned().unwrap_or("SHIFT".to_string()));
                }
                let final_key = if !mappings.contains_key(&key_name) {
                    if let Some(mc) = map_char_key(&key_name, false) {
                        mc.to_uppercase()
                    } else {
                        base_key.to_uppercase()
                    }
                } else {
                    base_key.to_uppercase()
                };
                chord.push(final_key);
                chord.join(" + ")
            };

            let mut should_process_press = false;

            if value == 1 { // Press
                if is_modifier {
                    modifiers.insert(key_name.clone());
                    
                    // Non-shift modifiers separate words
                    if key_name != "LEFTSHIFT" && key_name != "RIGHTSHIFT" {
                        let mut s = state.borrow_mut();
                        s.label = None;
                        s.text.clear();
                        if let Some(id) = s.timeout_id.take() {
                            id.remove();
                        }
                    }
                } else {
                    let mut sp = spam_state.borrow_mut();
                    if let Some(ref mut spam) = *sp {
                        if spam.key_name == key_name {
                            spam.count += 1;
                        } else {
                            if let Some(id) = spam.timeout_id.take() {
                                id.remove();
                            }
                            *sp = Some(SpamState {
                                key_name: key_name.clone(),
                                display_name: display_name.clone(),
                                count: 1,
                                label: None,
                                first_press_time: std::time::Instant::now(),
                                timeout_id: None,
                                repeat_ticks: 0,
                            });
                        }
                    } else {
                        *sp = Some(SpamState {
                            key_name: key_name.clone(),
                            display_name: display_name.clone(),
                            count: 1,
                            label: None,
                            first_press_time: std::time::Instant::now(),
                            timeout_id: None,
                            repeat_ticks: 0,
                        });
                    }
                    should_process_press = true;
                }
            } else if value == 2 { // Repeat
                if !is_modifier {
                    let mut sp = spam_state.borrow_mut();
                    if let Some(ref mut spam) = *sp {
                        if spam.key_name == key_name {
                            let elapsed = spam.first_press_time.elapsed().as_millis();
                            if elapsed >= spam_hold_ms {
                                spam.repeat_ticks += 1;
                                if spam.repeat_ticks % 3 == 0 {
                                    spam.count += 1;
                                    should_process_press = true;
                                }
                            }
                        }
                    }
                }
            } else if value == 0 { // Release
                if is_modifier {
                    modifiers.remove(&key_name);
                }
            }

            if should_process_press {
                let mut is_spam = false;
                {
                    let sp = spam_state.borrow();
                    if let Some(ref spam) = *sp {
                        if spam.count >= spam_threshold {
                            is_spam = true;
                        }
                    }
                }

                if is_spam {
                    let mut sp = spam_state.borrow_mut();
                    let spam = sp.as_mut().unwrap();
                    let text = format!("{} ({})", spam.display_name, spam.count);

                    if spam.count == spam_threshold && value == 1 {
                        // First time hitting the threshold
                        if is_word_key {
                            let mut s = state.borrow_mut();
                            let chars_to_remove = spam_threshold - 1;
                            let chars: Vec<char> = s.text.chars().collect();
                            if chars.len() >= chars_to_remove {
                                let new_len = chars.len() - chars_to_remove;
                                s.text = chars[..new_len].iter().collect();
                                if s.text.is_empty() {
                                    if let Some(lbl) = s.label.take() {
                                        container_clone.remove(&lbl);
                                    }
                                    if let Some(id) = s.timeout_id.take() {
                                        id.remove();
                                    }
                                } else {
                                    if let Some(lbl) = &s.label {
                                        lbl.set_label(&s.text);
                                    }
                                }
                            }
                        } else {
                            let mut labels_to_remove = vec![];
                            let mut child = container_clone.last_child();
                            let mut removed_count = 0;
                            while let Some(c) = child {
                                let prev = c.prev_sibling();
                                if let Ok(lbl) = c.clone().downcast::<Label>() {
                                    if lbl.label() == spam.display_name {
                                        labels_to_remove.push(lbl.clone());
                                        removed_count += 1;
                                        if removed_count == spam_threshold - 1 {
                                            break;
                                        }
                                    }
                                }
                                child = prev;
                            }
                            for lbl in labels_to_remove {
                                container_clone.remove(&lbl);
                            }
                        }

                        let label = Label::builder().label(&text).build();
                        container_clone.append(&label);
                        spam.label = Some(label.clone());
                    } else {
                        if let Some(lbl) = &spam.label {
                            lbl.set_label(&text);
                        } else {
                            let label = Label::builder().label(&text).build();
                            container_clone.append(&label);
                            spam.label = Some(label.clone());
                        }
                    }

                    // Reset fade out timeout
                    if let Some(id) = spam.timeout_id.take() {
                        id.remove();
                    }
                    if let Some(lbl) = &spam.label {
                        let label_clone = lbl.clone();
                        let container_clone_inner = container_clone.clone();
                        let sp_clone = spam_state.clone();
                        let key_name_clone = spam.key_name.clone();
                        let id = glib::timeout_add_local_once(Duration::from_secs(2), move || {
                            container_clone_inner.remove(&label_clone);
                            let mut sp_inner = sp_clone.borrow_mut();
                            if let Some(ref mut spam_inner) = *sp_inner {
                                if spam_inner.key_name == key_name_clone && spam_inner.label.as_ref() == Some(&label_clone) {
                                    spam_inner.label = None;
                                }
                            }
                        });
                        spam.timeout_id = Some(id);
                    }

                } else if value == 1 { // Not spam yet, and it's a press
                    if is_word_key {
                        let mut s = state.borrow_mut();
                        
                        if let Some(label) = s.label.clone() {
                            s.text.push_str(&display_name);
                            label.set_label(&s.text);
                            if let Some(id) = s.timeout_id.take() {
                                id.remove();
                            }
                        } else {
                            s.text = display_name.clone();
                            let label = Label::builder().label(&s.text).build();
                            container_clone.append(&label);
                            s.label = Some(label);
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
                            }
                        });
                        s.timeout_id = Some(id);
                    } else if key_name == "SPACE" || key_name == "BACKSPACE" {
                        let mut s = state.borrow_mut();
                        if s.label.is_some() {
                            s.label = None;
                            s.text.clear();
                            if let Some(id) = s.timeout_id.take() {
                                id.remove();
                            }
                        } else {
                            let label = Label::builder().label(&display_name).build();
                            container_clone.append(&label);
                            let sc = label.clone();
                            let cc = container_clone.clone();
                            glib::timeout_add_local_once(Duration::from_secs(2), move || {
                                cc.remove(&sc);
                            });
                        }
                    } else {
                        // Chord
                        {
                            let mut s = state.borrow_mut();
                            s.label = None;
                            s.text.clear();
                            if let Some(id) = s.timeout_id.take() {
                                id.remove();
                            }
                        }

                        let label = Label::builder().label(&display_name).build();
                        container_clone.append(&label);

                        let label_clone = label.clone();
                        let container_clone_inner = container_clone.clone();
                        glib::timeout_add_local_once(Duration::from_secs(2), move || {
                            container_clone_inner.remove(&label_clone);
                        });
                    }
                }
            }

            // Enforce max keys limit
            if config.appearance.max_keys > 0 {
                let mut count: usize = 0;
                let mut child = container_clone.first_child();
                while let Some(c) = child.clone() {
                    count += 1;
                    child = c.next_sibling();
                }
                let mut to_remove = count.saturating_sub(config.appearance.max_keys);
                let mut child = container_clone.first_child();
                while to_remove > 0 {
                    if let Some(c) = child {
                        let next = c.next_sibling();
                        
                        // Clean up references
                        if let Ok(lbl) = c.clone().downcast::<Label>() {
                            let mut s = state.borrow_mut();
                            if s.label.as_ref() == Some(&lbl) {
                                s.label = None;
                                s.text.clear();
                                if let Some(id) = s.timeout_id.take() {
                                    id.remove();
                                }
                            }
                            let mut sp = spam_state.borrow_mut();
                            if let Some(ref mut spam) = *sp {
                                if spam.label.as_ref() == Some(&lbl) {
                                    spam.label = None;
                                    if let Some(id) = spam.timeout_id.take() {
                                        id.remove();
                                    }
                                }
                            }
                        }
                        
                        container_clone.remove(&c);
                        child = next;
                        to_remove -= 1;
                    } else {
                        break;
                    }
                }
            }
        }
        glib::ControlFlow::Continue
    });
}
