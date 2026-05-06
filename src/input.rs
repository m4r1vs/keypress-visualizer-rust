use evdev::Device;
use glob::glob;
use std::sync::mpsc::Sender;

pub fn find_keyboard_device() -> Option<String> {
    let pattern = "/dev/input/by-id/usb-*event-kbd";
    if let Ok(paths) = glob(pattern) {
        for entry in paths {
            if let Ok(path) = entry {
                return Some(path.to_string_lossy().to_string());
            }
        }
    }
    None
}

pub fn spawn_input_thread(device_path: String, tx: Sender<(String, i32)>) {
    std::thread::spawn(move || {
        if let Err(e) = run_input_loop(device_path, tx) {
            eprintln!("Error in input loop: {}", e);
        }
    });
}

fn run_input_loop(device_path: String, tx: Sender<(String, i32)>) -> Result<(), Box<dyn std::error::Error>> {
    let mut device = Device::open(&device_path)?;
    println!("Opened device: {}", device_path);

    loop {
        for event in device.fetch_events()? {
            if let evdev::EventSummary::Key(_, key, value) = event.destructure() {
                // value: 0 = Release, 1 = Press, 2 = Repeat
                let key_name = format!("{:?}", key);
                let clean_name = key_name.trim_start_matches("KEY_").to_string();
                if tx.send((clean_name, value)).is_err() {
                    return Ok(()); // Main thread closed
                }
            }
        }
    }
}
