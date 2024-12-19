use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub const GLOBAL_CONFIG_FILE: &str = "global_config.json";

pub fn get_config_path(app_handle: &AppHandle) -> PathBuf {
    let mut path = app_handle
        .path()
        .app_config_dir()
        .expect("Could not get app config dir");
    path.push(GLOBAL_CONFIG_FILE);
    path
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub region: Option<Region>,
    pub monitor: i8,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            region: None,
            monitor: 0,
        }
    }
}

impl Config {
    pub fn load(app: &AppHandle) -> Self {
        let config_path = get_config_path(app);

        let config = if !config_path.exists() {
            let parent = config_path.parent().unwrap();
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Could not create config dir");
            }

            let data = serde_json::to_string(&Config::default())
                .expect("Could not serialize default config");
            fs::write(config_path, &data).expect("Could not write config.json");
            data
        } else {
            fs::read_to_string(config_path).expect("Could not read config.json")
        };
        let config =
            serde_json::from_str::<Config>(&config).expect("Could not deserialize config.json");
        config
    }

    pub fn save(&self, app: &AppHandle) {
        let config = serde_json::to_string(self).expect("Could not stringify config");
        fs::write(get_config_path(app), &config).expect("Could not write config.json");
    }
}

pub struct ConfigState(pub Mutex<Config>);
