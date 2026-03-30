use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Whether the active palette is a preset or a custom user-created palette.
/// Serialized as `snake_case`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PaletteType {
    /// A built-in preset palette.
    Preset,
    /// A user-created custom palette.
    Custom,
}

/// Available preset color palettes. Serialized as `kebab-case`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PresetPalette {
    /// Catppuccin color scheme.
    Catppuccin,
    /// Tokyo Night color scheme.
    TokyoNight,
    /// Everforest color scheme.
    Everforest,
}

/// A user-created custom color palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    /// Unique identifier for the palette.
    pub id: Uuid,
    /// The user who owns this palette.
    pub user_id: Uuid,
    /// Display name of the palette.
    pub name: String,
    /// Base seed colors used to generate the full palette (e.g., hex strings).
    pub seed_colors: Vec<String>,
    /// Generated design tokens derived from the seed colors.
    pub generated_tokens: Value,
    /// Timestamp when the palette was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the palette was last updated.
    pub updated_at: DateTime<Utc>,
}

/// A user's appearance preferences including their active color palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearancePreferences {
    /// Unique identifier for the preferences record.
    pub id: Uuid,
    /// The user these preferences belong to.
    pub user_id: Uuid,
    /// Whether the user is using a preset or custom palette.
    pub active_palette_type: PaletteType,
    /// The active preset palette, if `active_palette_type` is `Preset`.
    pub active_preset_palette: Option<PresetPalette>,
    /// The ID of the active custom palette, if `active_palette_type` is `Custom`.
    pub active_custom_palette_id: Option<Uuid>,
}

/// Request payload for creating a new custom palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePaletteRequest {
    /// Display name of the palette.
    pub name: String,
    /// Base seed colors used to generate the full palette.
    pub seed_colors: Vec<String>,
}

/// Request payload for updating an existing custom palette. All fields are optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePaletteRequest {
    /// Updated palette name.
    pub name: Option<String>,
    /// Updated seed colors.
    pub seed_colors: Option<Vec<String>>,
}

/// Request payload for switching the active palette.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateActivePaletteRequest {
    /// Whether to activate a preset or custom palette.
    pub active_palette_type: PaletteType,
    /// The preset palette to activate, if `active_palette_type` is `Preset`.
    pub active_preset_palette: Option<PresetPalette>,
    /// The custom palette ID to activate, if `active_palette_type` is `Custom`.
    pub active_custom_palette_id: Option<Uuid>,
}
