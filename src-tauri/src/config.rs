use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CopyMode {
    Translated,
    Original,
    Both,
}

impl Default for CopyMode {
    fn default() -> Self {
        Self::Translated
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub enabled: bool,
    pub opacity: f64,
    pub font_size: u32,
    pub font_color: String,
    pub background_color: String,
    pub border_radius: u32,
    pub shadow: bool,
    pub min_length: usize,
    pub english_only: bool,
    #[serde(default)]
    pub copy_mode: CopyMode,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            opacity: 0.9,
            font_size: 14,
            font_color: "#ffffff".to_string(),
            background_color: "#222222".to_string(),
            border_radius: 8,
            shadow: true,
            min_length: 4,
            english_only: true,
            copy_mode: CopyMode::default(),
        }
    }
}

fn config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("selection-translator");
    config_dir
}

fn config_file_path() -> PathBuf {
    config_path().join("config.json")
}

pub fn load_config() -> Config {
    let path = config_file_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let dir = config_path();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(config_file_path(), json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.enabled);
        assert_eq!(config.opacity, 0.9);
        assert_eq!(config.font_size, 14);
        assert_eq!(config.min_length, 4);
        assert!(config.english_only);
    }

    #[test]
    fn test_serialize_roundtrip() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_camel_case_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("fontSize"));
        assert!(json.contains("fontColor"));
        assert!(json.contains("backgroundColor"));
        assert!(json.contains("borderRadius"));
        assert!(json.contains("minLength"));
        assert!(json.contains("englishOnly"));
    }

    #[test]
    fn test_deserialize_partial_json() {
        let json = r#"{"enabled": false, "opacity": 0.5}"#;
        let result: Result<Config, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load_config() {
        let temp_dir = env::temp_dir().join("selection-translator-test");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let config_file = temp_dir.join("config.json");
        let config = Config {
            enabled: false,
            opacity: 0.7,
            ..Config::default()
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        fs::write(&config_file, &json).unwrap();

        let loaded: Config = serde_json::from_str(&fs::read_to_string(&config_file).unwrap()).unwrap();
        assert_eq!(config, loaded);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
