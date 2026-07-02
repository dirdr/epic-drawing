use wasm_bindgen::prelude::*;

pub mod cache;
pub mod config;
pub mod contour;
pub mod fourier;
pub mod linking;
pub mod morphology;
pub mod observer;
pub mod pipeline;

pub use cache::{ImageData, ImagesResponse, sample_contour_points};
pub use config::{ImageConfig, ImagesConfig, parse_config};
pub use fourier::{Coefficient, EquationData, compute_coefficients};
pub use pipeline::{Pipeline, PipelineConfig};

#[cfg(not(target_arch = "wasm32"))]
pub use config::load_config;

#[cfg(not(target_arch = "wasm32"))]
pub use observer::FileObserver;

#[wasm_bindgen]
pub fn get_all_images() -> Result<JsValue, JsValue> {
    let images = cache::get_cached_images()
        .map_err(|e| JsValue::from_str(&format!("Cache error: {}", e)))?;
    serde_wasm_bindgen::to_value(images)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[wasm_bindgen]
pub fn get_image_names() -> Result<JsValue, JsValue> {
    let images = cache::get_cached_images()
        .map_err(|e| JsValue::from_str(&format!("Cache error: {}", e)))?;
    let names: Vec<&str> = images.images.iter().map(|img| img.name.as_str()).collect();
    serde_wasm_bindgen::to_value(&names)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}
