//! Models for user-defined appearance color palettes.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

/// Generated RGB token values for a complete color palette.
///
/// These values map directly to the CSS variables consumed by the web client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPaletteTokens {
    /// Base dark neutral color as `r, g, b`.
    pub black: String,
    /// Base light neutral color as `r, g, b`.
    pub white: String,
    /// Strong green accent color as `r, g, b`.
    pub green_100: String,
    /// Medium green accent color as `r, g, b`.
    pub green_80: String,
    /// Soft green accent color as `r, g, b`.
    pub green_60: String,
    /// Strong red accent color as `r, g, b`.
    pub red_100: String,
    /// Medium red accent color as `r, g, b`.
    pub red_80: String,
    /// Soft red accent color as `r, g, b`.
    pub red_60: String,
    /// Strong yellow accent color as `r, g, b`.
    pub yellow_100: String,
    /// Medium yellow accent color as `r, g, b`.
    pub yellow_80: String,
    /// Soft yellow accent color as `r, g, b`.
    pub yellow_60: String,
    /// Strong blue accent color as `r, g, b`.
    pub blue_100: String,
    /// Medium blue accent color as `r, g, b`.
    pub blue_80: String,
    /// Soft blue accent color as `r, g, b`.
    pub blue_60: String,
    /// Strong magenta accent color as `r, g, b`.
    pub magenta_100: String,
    /// Medium magenta accent color as `r, g, b`.
    pub magenta_80: String,
    /// Soft magenta accent color as `r, g, b`.
    pub magenta_60: String,
    /// Strong cyan accent color as `r, g, b`.
    pub cyan_100: String,
    /// Medium cyan accent color as `r, g, b`.
    pub cyan_80: String,
    /// Soft cyan accent color as `r, g, b`.
    pub cyan_60: String,
}

/// Represents a user-created color palette persisted in the database.
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct UserColorPalette {
    /// Unique palette identifier.
    pub id: Uuid,
    /// Owner user identifier.
    pub user_id: Uuid,
    /// Human-readable palette name.
    pub name: String,
    /// Seed green color in 6-digit hex format.
    pub green_seed_hex: String,
    /// Seed red color in 6-digit hex format.
    pub red_seed_hex: String,
    /// Seed yellow color in 6-digit hex format.
    pub yellow_seed_hex: String,
    /// Seed blue color in 6-digit hex format.
    pub blue_seed_hex: String,
    /// Seed magenta color in 6-digit hex format.
    pub magenta_seed_hex: String,
    /// Seed cyan color in 6-digit hex format.
    pub cyan_seed_hex: String,
    /// Generated token map persisted as JSON.
    pub generated_tokens: Value,
    /// Algorithm version used to generate `generated_tokens`.
    pub generation_version: i32,
    /// Timestamp when the palette was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the palette was last updated.
    pub updated_at: DateTime<Utc>,
}
