//! Request and response payloads for appearance endpoints.
//!
//! This module contains data structures used for serializing and deserializing
//! request/response bodies in appearance handlers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::models::color_palette::GeneratedPaletteTokens;
use crate::validators::color_hex::validate_hex_color;

/// Request body for creating a custom user color palette.
///
/// See [`create_custom_palette`](super::handlers::create_custom_palette) for
/// the handler that processes this request.
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCustomPaletteRequest {
    /// User-defined palette name.
    #[validate(length(min = 1, max = 50, message = "Palette name is required"))]
    pub name: String,

    /// Base green accent in `#RRGGBB` format.
    #[validate(custom(function = "validate_hex_color"))]
    pub green_seed_hex: String,

    /// Base red accent in `#RRGGBB` format.
    #[validate(custom(function = "validate_hex_color"))]
    pub red_seed_hex: String,

    /// Base yellow accent in `#RRGGBB` format.
    #[validate(custom(function = "validate_hex_color"))]
    pub yellow_seed_hex: String,

    /// Base blue accent in `#RRGGBB` format.
    #[validate(custom(function = "validate_hex_color"))]
    pub blue_seed_hex: String,

    /// Base magenta accent in `#RRGGBB` format.
    #[validate(custom(function = "validate_hex_color"))]
    pub magenta_seed_hex: String,

    /// Base cyan accent in `#RRGGBB` format.
    #[validate(custom(function = "validate_hex_color"))]
    pub cyan_seed_hex: String,
}

/// Request body for setting a user's active palette selection.
///
/// See [`set_active_palette`](super::handlers::set_active_palette) for the
/// handler that processes this request.
#[derive(Debug, Deserialize, Validate)]
pub struct SetActivePaletteRequest {
    /// Active palette source (`preset` or `custom`).
    #[validate(length(min = 1, message = "Palette type is required"))]
    pub palette_type: String,

    /// Built-in preset name when `palette_type` is `preset`.
    pub preset_palette: Option<String>,

    /// User palette identifier when `palette_type` is `custom`.
    pub custom_palette_id: Option<Uuid>,
}

/// Response shape describing the user's active palette selection.
#[derive(Debug, Serialize)]
pub struct ActivePaletteSelectionResponse {
    /// Active palette source (`preset` or `custom`).
    pub palette_type: String,
    /// Active built-in preset value when source is `preset`.
    pub preset_palette: Option<String>,
    /// Active custom palette ID when source is `custom`.
    pub custom_palette_id: Option<Uuid>,
}

/// Custom palette payload returned by appearance endpoints.
#[derive(Debug, Serialize)]
pub struct CustomPaletteResponse {
    /// Unique custom palette identifier.
    pub id: Uuid,
    /// User-defined palette name.
    pub name: String,
    /// Base green accent in `#RRGGBB` format.
    pub green_seed_hex: String,
    /// Base red accent in `#RRGGBB` format.
    pub red_seed_hex: String,
    /// Base yellow accent in `#RRGGBB` format.
    pub yellow_seed_hex: String,
    /// Base blue accent in `#RRGGBB` format.
    pub blue_seed_hex: String,
    /// Base magenta accent in `#RRGGBB` format.
    pub magenta_seed_hex: String,
    /// Base cyan accent in `#RRGGBB` format.
    pub cyan_seed_hex: String,
    /// Generated 100/80/60 RGB token map.
    pub generated_tokens: GeneratedPaletteTokens,
    /// Generator algorithm version used for `generated_tokens`.
    pub generation_version: i32,
    /// Timestamp when the custom palette was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the custom palette was last updated.
    pub updated_at: DateTime<Utc>,
}

/// Response body for loading appearance settings.
///
/// See [`get_appearance`](super::handlers::get_appearance) for the handler
/// that produces this response.
#[derive(Debug, Serialize)]
pub struct GetAppearanceResponse {
    /// User's active palette selection.
    pub active_palette: ActivePaletteSelectionResponse,
    /// All custom palettes owned by the user.
    pub custom_palettes: Vec<CustomPaletteResponse>,
}

/// Response body for successful custom palette creation.
///
/// See [`create_custom_palette`](super::handlers::create_custom_palette) for
/// the handler that produces this response.
#[derive(Debug, Serialize)]
pub struct CreateCustomPaletteResponse {
    /// Success message.
    pub message: String,
    /// Newly created custom palette.
    pub palette: CustomPaletteResponse,
    /// Updated active palette selection.
    pub active_palette: ActivePaletteSelectionResponse,
}

/// Response body for successful active-palette updates.
///
/// See [`set_active_palette`](super::handlers::set_active_palette) for the
/// handler that produces this response.
#[derive(Debug, Serialize)]
pub struct SetActivePaletteResponse {
    /// Success message.
    pub message: String,
    /// Updated active palette selection.
    pub active_palette: ActivePaletteSelectionResponse,
}
