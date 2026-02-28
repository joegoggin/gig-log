use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaletteType {
    Preset,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PresetPalette {
    Catppuccin,
    TokyoNight,
    Everforest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub seed_colors: Vec<String>,
    pub generated_tokens: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearancePreferences {
    pub id: Uuid,
    pub user_id: Uuid,
    pub active_palette_type: PaletteType,
    pub active_preset_palette: Option<PresetPalette>,
    pub active_custom_palette_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaletteRequest {
    pub name: String,
    pub seed_colors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePaletteRequest {
    pub name: Option<String>,
    pub seed_colors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateActivePaletteRequest {
    pub active_palette_type: PaletteType,
    pub active_preset_palette: Option<PresetPalette>,
    pub active_custom_palette_id: Option<Uuid>,
}
