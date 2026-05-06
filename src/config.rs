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

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub mappings: HashMap<String, String>,
    #[serde(default)]
    pub appearance: AppearanceConfig,
}

pub fn load_config() -> Config {
    let config_str = fs::read_to_string("default_config.toml").unwrap_or_else(|_| {
        eprintln!("Warning: Could not read default_config.toml, using empty mappings.");
        "[mappings]".to_string()
    });
    toml::from_str(&config_str).unwrap_or_else(|e| {
        eprintln!(
            "Warning: Error parsing config: {}, using empty mappings.",
            e
        );
        Config {
            mappings: HashMap::new(),
            appearance: AppearanceConfig::default(),
        }
    })
}