use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Label, Orientation};
use gtk4_layer_shell::{Layer, LayerShell};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::config::{AppearanceConfig, Config, load_config};
use crate::input;
use crate::tray;
use crate::utils::map_char_key;

pub fn build_ui(app: &Application) {
    let config = load_config();

    let (tx_quit, rx_quit) = std::sync::mpsc::channel::<()>();
    if config.show_in_tray {
        tray::spawn_tray(tx_quit);
    }

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Keypress Visualizer")
        .build();

    setup_layer_shell(&window, &config.appearance);

    let container = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .halign(gtk4::Align::Center)
        .build();

    window.set_child(Some(&container));

    setup_css(&config.appearance);

    window.present();

    let (tx, rx) = std::sync::mpsc::channel::<(String, i32)>();
    start_input_thread(tx);

    setup_event_loop(rx, rx_quit, container, config, app.clone());
}

fn setup_layer_shell(window: &ApplicationWindow, appearance: &AppearanceConfig) {
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("keypress-visualizer"));

    let anchor_str = appearance.anchor.to_lowercase();
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

    let mut m_top = appearance.margin_y;
    let m_bottom = appearance.margin_y;
    let mut m_left = appearance.margin_x;
    let m_right = appearance.margin_x;

    if appearance.pos_x_pct > 0.0 {
        m_left += (width * appearance.pos_x_pct / 100.0) as i32;
    }
    if appearance.pos_y_pct > 0.0 {
        m_top += (height * appearance.pos_y_pct / 100.0) as i32;
    }

    if top {
        window.set_margin(gtk4_layer_shell::Edge::Top, m_top);
    }
    if bottom {
        window.set_margin(gtk4_layer_shell::Edge::Bottom, m_bottom);
    }
    if left {
        window.set_margin(gtk4_layer_shell::Edge::Left, m_left);
    }
    if right {
        window.set_margin(gtk4_layer_shell::Edge::Right, m_right);
    }
}

fn setup_css(appearance: &AppearanceConfig) {
    let provider = gtk4::CssProvider::new();
    let css_path = &appearance.custom_css;
    match std::fs::read_to_string(css_path) {
        Ok(css) => provider.load_from_data(&css),
        Err(e) => {
            eprintln!("Warning: Failed to load CSS from {}: {}", css_path.display(), e);
        }
    }

    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn start_input_thread(tx: std::sync::mpsc::Sender<(String, i32)>) {
    if let Some(device_path) = input::find_keyboard_device() {
        println!("Found keyboard: {}", device_path);
        input::spawn_input_thread(device_path, tx);
    } else {
        eprintln!("No keyboard device found matching the pattern!");
    }
}

struct WordState {
    label: Option<Label>,
    text: String,
    generation: u64,
}

struct SpamState {
    key_name: String,
    display_name: String,
    count: usize,
    label: Option<Label>,
    first_press_time: Instant,
    repeat_ticks: usize,
    generation: u64,
}

struct AppState {
    container: GtkBox,
    config: Config,
    modifiers: HashSet<String>,
    word_state: WordState,
    spam_state: Option<SpamState>,
    next_generation: u64,
}

impl AppState {
    fn new(container: GtkBox, config: Config) -> Self {
        Self {
            container,
            config,
            modifiers: HashSet::new(),
            word_state: WordState {
                label: None,
                text: String::new(),
                generation: 0,
            },
            spam_state: None,
            next_generation: 0,
        }
    }

    fn next_gen(&mut self) -> u64 {
        self.next_generation += 1;
        self.next_generation
    }
}

fn setup_event_loop(
    rx: std::sync::mpsc::Receiver<(String, i32)>,
    rx_quit: std::sync::mpsc::Receiver<()>,
    container: GtkBox,
    config: Config,
    app: Application,
) {
    let state = Rc::new(RefCell::new(AppState::new(container, config)));

    glib::timeout_add_local(Duration::from_millis(10), move || {
        if let Ok(()) = rx_quit.try_recv() {
            app.quit();
            return glib::ControlFlow::Break;
        }
        while let Ok((key_name, value)) = rx.try_recv() {
            process_key_event(state.clone(), key_name, value);
        }
        glib::ControlFlow::Continue
    });
}

fn process_key_event(state_ref: Rc<RefCell<AppState>>, key_name: String, value: i32) {
    let mut state = state_ref.borrow_mut();

    let is_modifier = matches!(
        key_name.as_str(),
        "LEFTCTRL"
            | "RIGHTCTRL"
            | "LEFTSHIFT"
            | "RIGHTSHIFT"
            | "LEFTALT"
            | "RIGHTALT"
            | "LEFTMETA"
            | "RIGHTMETA"
    );

    let has_shift = state.modifiers.contains("LEFTSHIFT") || state.modifiers.contains("RIGHTSHIFT");
    let mut active_mods = Vec::new();
    for m in &["CTRL", "ALT", "META"] {
        if state.modifiers.contains(&format!("LEFT{}", m))
            || state.modifiers.contains(&format!("RIGHT{}", m))
        {
            let mod_name = if state.modifiers.contains(&format!("LEFT{}", m)) {
                format!("LEFT{}", m)
            } else {
                format!("RIGHT{}", m)
            };
            active_mods.push(
                state
                    .config
                    .mappings
                    .get(&mod_name)
                    .cloned()
                    .unwrap_or(mod_name),
            );
        }
    }

    let base_key = state
        .config
        .mappings
        .get(&key_name)
        .cloned()
        .unwrap_or(key_name.clone());
    let mut mapped_char = None;
    if !state.config.mappings.contains_key(&key_name) {
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
            if has_shift {
                base_key.to_uppercase()
            } else {
                base_key.to_lowercase()
            }
        }
    } else if key_name == "SPACE" {
        state
            .config
            .mappings
            .get("SPACE")
            .cloned()
            .unwrap_or_else(|| "SPC".to_string())
    } else if key_name == "BACKSPACE" {
        state
            .config
            .mappings
            .get("BACKSPACE")
            .cloned()
            .unwrap_or_else(|| "⌫".to_string())
    } else {
        let mut chord = active_mods.clone();
        if has_shift {
            let shift_name = if state.modifiers.contains("LEFTSHIFT") {
                "LEFTSHIFT"
            } else {
                "RIGHTSHIFT"
            };
            chord.push(
                state
                    .config
                    .mappings
                    .get(shift_name)
                    .cloned()
                    .unwrap_or_else(|| "SHIFT".to_string()),
            );
        }
        let final_key = if !state.config.mappings.contains_key(&key_name) {
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

    if value == 1 {
        if is_modifier {
            state.modifiers.insert(key_name.clone());
            if key_name != "LEFTSHIFT" && key_name != "RIGHTSHIFT" {
                state.word_state.label = None;
                state.word_state.text.clear();
                state.word_state.generation = state.next_gen();
            }
        } else {
            if let Some(ref mut spam) = state.spam_state {
                if spam.key_name == key_name {
                    spam.count += 1;
                } else {
                    let next_gen = state.next_gen();
                    state.spam_state = Some(SpamState {
                        key_name: key_name.clone(),
                        display_name: display_name.clone(),
                        count: 1,
                        label: None,
                        first_press_time: Instant::now(),
                        repeat_ticks: 0,
                        generation: next_gen,
                    });
                }
            } else {
                let next_gen = state.next_gen();
                state.spam_state = Some(SpamState {
                    key_name: key_name.clone(),
                    display_name: display_name.clone(),
                    count: 1,
                    label: None,
                    first_press_time: Instant::now(),
                    repeat_ticks: 0,
                    generation: next_gen,
                });
            }
            should_process_press = true;
        }
    } else if value == 2 {
        if !is_modifier {
            let spam_hold_ms = state.config.appearance.spam_hold_ms as u128;
            if let Some(ref mut spam) = state.spam_state {
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
    } else if value == 0 {
        if is_modifier {
            state.modifiers.remove(&key_name);
        }
    }

    if should_process_press {
        handle_keypress_ui(
            state_ref.clone(),
            &mut state,
            key_name,
            display_name,
            is_word_key,
            value,
        );
    }

    enforce_max_keys(&mut state);
}

fn handle_keypress_ui(
    state_ref: Rc<RefCell<AppState>>,
    state: &mut std::cell::RefMut<AppState>,
    key_name: String,
    display_name: String,
    is_word_key: bool,
    value: i32,
) {
    let spam_threshold = state.config.appearance.spam_threshold;

    let mut is_spam = false;
    if let Some(ref spam) = state.spam_state {
        if spam.count >= spam_threshold {
            is_spam = true;
        }
    }

    if is_spam {
        handle_spam_ui(
            state_ref,
            state,
            key_name,
            display_name,
            is_word_key,
            value,
            spam_threshold,
        );
    } else if value == 1 {
        handle_regular_ui(state_ref, state, key_name, display_name, is_word_key);
    }
}

fn handle_spam_ui(
    state_ref: Rc<RefCell<AppState>>,
    state: &mut std::cell::RefMut<AppState>,
    _key_name: String,
    _display_name: String,
    is_word_key: bool,
    value: i32,
    spam_threshold: usize,
) {
    let spam_count = state.spam_state.as_ref().unwrap().count;
    let spam_display_name = state.spam_state.as_ref().unwrap().display_name.clone();
    let text = format!("{} ({})", spam_display_name, spam_count);

    if spam_count == spam_threshold && value == 1 {
        if is_word_key {
            let chars_to_remove = spam_threshold - 1;
            let chars: Vec<char> = state.word_state.text.chars().collect();
            if chars.len() >= chars_to_remove {
                let new_len = chars.len() - chars_to_remove;
                state.word_state.text = chars[..new_len].iter().collect();
                if state.word_state.text.is_empty() {
                    if let Some(lbl) = state.word_state.label.take() {
                        if lbl.parent().is_some() {
                            state.container.remove(&lbl);
                        }
                    }
                    state.word_state.generation = state.next_gen();
                } else if let Some(lbl) = &state.word_state.label {
                    lbl.set_label(&state.word_state.text);
                }
            }
        } else {
            let mut labels_to_remove = vec![];
            let mut child = state.container.last_child();
            let mut removed_count = 0;
            while let Some(c) = child {
                let prev = c.prev_sibling();
                if let Ok(lbl) = c.clone().downcast::<Label>() {
                    if lbl.label() == spam_display_name {
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
                if lbl.parent().is_some() {
                    state.container.remove(&lbl);
                }
            }
        }

        let label = Label::builder().label(&text).build();
        state.container.append(&label);
        state.spam_state.as_mut().unwrap().label = Some(label.clone());
    } else {
        let mut new_label = None;
        {
            let spam = state.spam_state.as_mut().unwrap();
            if let Some(lbl) = &spam.label {
                lbl.set_label(&text);
            } else {
                new_label = Some(Label::builder().label(&text).build());
            }
        }
        if let Some(label) = new_label {
            state.container.append(&label);
            state.spam_state.as_mut().unwrap().label = Some(label);
        }
    }

    let container_clone = state.container.clone();
    let (generation, spam_label) = {
        let next_gen = state.next_gen();
        let spam = state.spam_state.as_mut().unwrap();
        spam.generation = next_gen;
        (spam.generation, spam.label.clone())
    };

    if let Some(lbl) = spam_label {
        let label_clone = lbl.clone();
        glib::timeout_add_local_once(Duration::from_secs(2), move || {
            if label_clone.parent().is_some() {
                container_clone.remove(&label_clone);
            }
            let mut s = state_ref.borrow_mut();
            if let Some(ref mut spam_inner) = s.spam_state {
                if spam_inner.generation == generation {
                    spam_inner.label = None;
                }
            }
        });
    }
}

fn handle_regular_ui(
    state_ref: Rc<RefCell<AppState>>,
    state: &mut std::cell::RefMut<AppState>,
    key_name: String,
    display_name: String,
    is_word_key: bool,
) {
    if is_word_key {
        if let Some(label) = state.word_state.label.clone() {
            state.word_state.text.push_str(&display_name);
            label.set_label(&state.word_state.text);
            state.word_state.generation = state.next_gen();
        } else {
            state.word_state.text = display_name.clone();
            let label = Label::builder().label(&state.word_state.text).build();
            state.container.append(&label);
            state.word_state.label = Some(label);
        }

        let label_clone = state.word_state.label.as_ref().unwrap().clone();
        let container_clone = state.container.clone();
        let generation = state.word_state.generation;
        glib::timeout_add_local_once(Duration::from_secs(2), move || {
            if label_clone.parent().is_some() {
                container_clone.remove(&label_clone);
            }
            let mut s = state_ref.borrow_mut();
            if s.word_state.generation == generation {
                s.word_state.label = None;
                s.word_state.text.clear();
            }
        });
    } else if key_name == "SPACE" || key_name == "BACKSPACE" {
        if state.word_state.label.is_some() {
            state.word_state.label = None;
            state.word_state.text.clear();
            state.word_state.generation = state.next_gen();
        } else {
            let label = Label::builder().label(&display_name).build();
            state.container.append(&label);
            let sc = label.clone();
            let cc = state.container.clone();
            glib::timeout_add_local_once(Duration::from_secs(2), move || {
                if sc.parent().is_some() {
                    cc.remove(&sc);
                }
            });
        }
    } else {
        // Chord
        state.word_state.label = None;
        state.word_state.text.clear();
        state.word_state.generation = state.next_gen();

        let label = Label::builder().label(&display_name).build();
        state.container.append(&label);

        let label_clone = label.clone();
        let container_clone = state.container.clone();
        glib::timeout_add_local_once(Duration::from_secs(2), move || {
            if label_clone.parent().is_some() {
                container_clone.remove(&label_clone);
            }
        });
    }
}

fn enforce_max_keys(state: &mut std::cell::RefMut<AppState>) {
    let max_keys = state.config.appearance.max_keys;
    if max_keys == 0 {
        return;
    }

    let mut count: usize = 0;
    let mut child = state.container.first_child();
    while let Some(c) = child.clone() {
        count += 1;
        child = c.next_sibling();
    }

    let mut to_remove = count.saturating_sub(max_keys);
    let mut child = state.container.first_child();
    while to_remove > 0 {
        if let Some(c) = child {
            let next = c.next_sibling();

            if let Ok(lbl) = c.clone().downcast::<Label>() {
                if state.word_state.label.as_ref() == Some(&lbl) {
                    state.word_state.label = None;
                    state.word_state.text.clear();
                    state.word_state.generation = state.next_gen();
                }
                let mut spam_needs_gen = false;
                if let Some(ref mut spam) = state.spam_state {
                    if spam.label.as_ref() == Some(&lbl) {
                        spam.label = None;
                        spam_needs_gen = true;
                    }
                }
                if spam_needs_gen {
                    let next_gen = state.next_gen();
                    state.spam_state.as_mut().unwrap().generation = next_gen;
                }
            }

            if c.parent().is_some() {
                state.container.remove(&c);
            }
            child = next;
            to_remove -= 1;
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_default_css() {
        let mut appearance = AppearanceConfig::default();
        appearance.font_size = 42;
        let css = generate_default_css(&appearance);

        // Should not start with backslash
        assert!(
            !css.trim_start().starts_with('\\'),
            "CSS should not contain literal backslashes from raw strings"
        );

        // Should contain the font size
        assert!(css.contains("font-size: 42px;"));

        // Should contain the basic GTK selectors
        assert!(css.contains("label {"));
        assert!(css.contains("window {"));
    }
}
