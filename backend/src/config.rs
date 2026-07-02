use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageConfig {
    pub name: String,
    pub file: String,
    pub num_points_sample: usize,
    pub max_coefficients: usize,
    pub target_size: f64,
    pub blur_sigma: Option<f32>,
    pub canny_low_threshold: Option<f32>,
    pub canny_high_threshold: Option<f32>,
    pub invert: Option<bool>,
    pub closing_iterations: Option<u8>,
    pub min_component_points: Option<usize>,
    pub max_components: Option<usize>,
    pub max_link_distance: Option<f32>,
    pub max_bridges: Option<usize>,
}

impl ImageConfig {
    pub fn blur_sigma(&self) -> f32 {
        self.blur_sigma.unwrap_or(1.0)
    }

    pub fn canny_low_threshold(&self) -> f32 {
        (self.canny_low_threshold.unwrap_or(20.0) / 100.0) * 255.0
    }

    pub fn canny_high_threshold(&self) -> f32 {
        (self.canny_high_threshold.unwrap_or(150.0) / 100.0) * 255.0
    }

    pub fn invert(&self) -> bool {
        self.invert.unwrap_or(false)
    }

    pub fn closing_iterations(&self) -> u8 {
        self.closing_iterations.unwrap_or(1)
    }

    pub fn min_component_points(&self) -> usize {
        self.min_component_points.unwrap_or(10)
    }

    pub fn max_components(&self) -> usize {
        self.max_components.unwrap_or(0)
    }

    pub fn max_link_distance(&self) -> f32 {
        self.max_link_distance.unwrap_or(25.0)
    }

    pub fn max_bridges(&self) -> usize {
        self.max_bridges.unwrap_or(50)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImagesConfig {
    pub image: Vec<ImageConfig>,
}

impl ImagesConfig {
    pub fn find_image(&self, name: &str) -> Option<&ImageConfig> {
        self.image.iter().find(|img| img.name == name)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        let mut names = std::collections::HashSet::new();
        for img in &self.image {
            if !names.insert(&img.name) {
                return Err(ConfigError::DuplicateName(img.name.clone()));
            }

            if img.num_points_sample == 0 {
                return Err(ConfigError::InvalidValue(format!(
                    "num_points must be > 0 for image '{}'",
                    img.name
                )));
            }

            if img.target_size <= 0.0 {
                return Err(ConfigError::InvalidValue(format!(
                    "target_size must be > 0.0 for image '{}'",
                    img.name
                )));
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),

    Parse(toml::de::Error),

    ImageNotFound(String),

    DuplicateName(String),

    InvalidValue(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "Failed to read config file: {}", err),
            ConfigError::Parse(err) => write!(f, "Failed to parse TOML: {}", err),
            ConfigError::ImageNotFound(path) => write!(f, "Image file not found: {}", path),
            ConfigError::DuplicateName(name) => write!(f, "Duplicate image name: {}", name),
            ConfigError::InvalidValue(msg) => write!(f, "Invalid configuration: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(err) => Some(err),
            ConfigError::Parse(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(err: std::io::Error) -> Self {
        ConfigError::Io(err)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(err: toml::de::Error) -> Self {
        ConfigError::Parse(err)
    }
}

pub fn parse_config(toml_str: &str) -> Result<ImagesConfig, ConfigError> {
    let config: ImagesConfig = toml::from_str(toml_str)?;
    config.validate()?;
    Ok(config)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_config(path: &str) -> Result<ImagesConfig, ConfigError> {
    let config_str = std::fs::read_to_string(path)?;
    parse_config(&config_str)
}
