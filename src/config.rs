use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Deserialize, Debug, Clone)]
#[serde(default)]
pub struct AppearanceConfig {
    pub font_size: u32,
    pub anchor: String,
    pub margin_x: i32,
    pub margin_y: i32,
    pub pos_x_pct: f64,
    pub pos_y_pct: f64,
    pub max_keys: usize,
    pub custom_css: std::path::PathBuf,
    pub spam_threshold: usize,
    pub spam_hold_ms: u64,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            font_size: 24,
            anchor: "bottom".to_string(),
            margin_x: 0,
            margin_y: 50,
            pos_x_pct: 0.0,
            pos_y_pct: 0.0,
            max_keys: 10,
            custom_css: std::path::PathBuf::from("default_style.css"),
            spam_threshold: 4,
            spam_hold_ms: 500,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default = "default_true")]
    pub show_in_tray: bool,
    #[serde(default)]
    pub mappings: HashMap<String, String>,
    #[serde(default)]
    pub appearance: AppearanceConfig,
}

pub fn load_config() -> Config {
    let mut config_path = std::path::PathBuf::from("default_config.toml");

    if !config_path.exists() {
        if let Ok(env_path) = std::env::var("KEYPRESS_VISUALIZER_CONFIG") {
            config_path = std::path::PathBuf::from(env_path);
        }
    }

    if !config_path.exists() {
        let etc_path = std::path::PathBuf::from("/etc/keypress-visualizer-rust/default_config.toml");
        if etc_path.exists() {
            config_path = etc_path;
        }
    }

    let config_str = fs::read_to_string(&config_path).unwrap_or_else(|_| {
        eprintln!(
            "Warning: Could not read {:?}, using empty mappings.",
            config_path
        );
        "[mappings]".to_string()
    });
    let mut config: Config = toml::from_str(&config_str).unwrap_or_else(|e| {
        eprintln!(
            "Warning: Error parsing config: {}, using empty mappings.",
            e
        );
        Config {
            show_in_tray: true,
            mappings: HashMap::new(),
            appearance: AppearanceConfig::default(),
        }
    });

    // If custom_css is default and doesn't exist in CWD, try relative to config
    if config.appearance.custom_css == std::path::PathBuf::from("default_style.css")
        && !config.appearance.custom_css.exists()
    {
        if let Some(parent) = config_path.parent() {
            let css_relative = parent.join("default_style.css");
            if css_relative.exists() {
                config.appearance.custom_css = css_relative;
            }
        }
    }

    config
}