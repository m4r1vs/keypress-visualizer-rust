use ksni::{Tray, MenuItem, menu::StandardItem};
use ksni::blocking::TrayMethods;
use std::sync::mpsc;

struct KeypressTray {
    tx_quit: mpsc::Sender<()>,
}

impl Tray for KeypressTray {
    fn id(&self) -> String {
        "keypress-visualizer".into()
    }

    fn icon_name(&self) -> String {
        "input-keyboard".into()
    }

    fn title(&self) -> String {
        "Keypress Visualizer".into()
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let tx = self.tx_quit.clone();
        vec![
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(move |_| {
                    let _ = tx.send(());
                }),
                ..Default::default()
            }.into(),
        ]
    }
}

pub fn spawn_tray(tx_quit: mpsc::Sender<()>) {
    let tray = KeypressTray { tx_quit };
    let _ = tray.spawn().expect("Failed to spawn tray icon");
}
