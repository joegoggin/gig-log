//! Color palette shade-generation utilities.
//!
//! This module generates a full palette token map from user-provided
//! role/accent seed colors by deriving lighter shades for each color group.

use crate::core::error::ApiError;
use crate::models::color_palette::GeneratedPaletteTokens;

/// Current algorithm version for generated palette tokens.
pub const PALETTE_GENERATION_VERSION: i32 = 2;

/// Base accent colors used to derive full palette token sets.
pub struct PaletteSeedColors<'a> {
    /// Base background color in `#RRGGBB` format.
    pub background_seed_hex: &'a str,
    /// Base text color in `#RRGGBB` format.
    pub text_seed_hex: &'a str,
    /// Base primary action color in `#RRGGBB` format.
    pub primary_seed_hex: &'a str,
    /// Base secondary action color in `#RRGGBB` format.
    pub secondary_seed_hex: &'a str,
    /// Base green accent in `#RRGGBB` format.
    pub green_seed_hex: &'a str,
    /// Base red accent in `#RRGGBB` format.
    pub red_seed_hex: &'a str,
    /// Base yellow accent in `#RRGGBB` format.
    pub yellow_seed_hex: &'a str,
    /// Base blue accent in `#RRGGBB` format.
    pub blue_seed_hex: &'a str,
    /// Base magenta accent in `#RRGGBB` format.
    pub magenta_seed_hex: &'a str,
    /// Base cyan accent in `#RRGGBB` format.
    pub cyan_seed_hex: &'a str,
}

/// Generates complete palette tokens from role/accent seed colors.
///
/// Each accent group receives:
/// - `100`: the original base color
/// - `80`: a lighter shade mixed 20% toward white
/// - `60`: a lighter shade mixed 40% toward white
///
/// Core interface roles (`background`, `text`, `primary`, `secondary`) are
/// generated from their corresponding seed colors and also backfill legacy
/// neutral tokens (`black`, `white`) for compatibility with existing styles.
///
/// # Arguments
///
/// - `seed_colors` - User-provided role/accent seed colors
///
/// # Errors
///
/// Returns [`ApiError::ValidationError`] if any seed color is not a valid
/// 6-digit hex value.
pub fn generate_palette_tokens(
    seed_colors: &PaletteSeedColors<'_>,
) -> Result<GeneratedPaletteTokens, ApiError> {
    let background = shade_triplet(seed_colors.background_seed_hex, 0.0)?;
    let text = shade_triplet(seed_colors.text_seed_hex, 0.0)?;

    Ok(GeneratedPaletteTokens {
        background: background.clone(),
        text: text.clone(),
        primary_100: shade_triplet(seed_colors.primary_seed_hex, 0.0)?,
        primary_80: shade_triplet(seed_colors.primary_seed_hex, 0.2)?,
        primary_60: shade_triplet(seed_colors.primary_seed_hex, 0.4)?,
        secondary_100: shade_triplet(seed_colors.secondary_seed_hex, 0.0)?,
        secondary_80: shade_triplet(seed_colors.secondary_seed_hex, 0.2)?,
        secondary_60: shade_triplet(seed_colors.secondary_seed_hex, 0.4)?,
        black: text,
        white: background,
        green_100: shade_triplet(seed_colors.green_seed_hex, 0.0)?,
        green_80: shade_triplet(seed_colors.green_seed_hex, 0.2)?,
        green_60: shade_triplet(seed_colors.green_seed_hex, 0.4)?,
        red_100: shade_triplet(seed_colors.red_seed_hex, 0.0)?,
        red_80: shade_triplet(seed_colors.red_seed_hex, 0.2)?,
        red_60: shade_triplet(seed_colors.red_seed_hex, 0.4)?,
        yellow_100: shade_triplet(seed_colors.yellow_seed_hex, 0.0)?,
        yellow_80: shade_triplet(seed_colors.yellow_seed_hex, 0.2)?,
        yellow_60: shade_triplet(seed_colors.yellow_seed_hex, 0.4)?,
        blue_100: shade_triplet(seed_colors.blue_seed_hex, 0.0)?,
        blue_80: shade_triplet(seed_colors.blue_seed_hex, 0.2)?,
        blue_60: shade_triplet(seed_colors.blue_seed_hex, 0.4)?,
        magenta_100: shade_triplet(seed_colors.magenta_seed_hex, 0.0)?,
        magenta_80: shade_triplet(seed_colors.magenta_seed_hex, 0.2)?,
        magenta_60: shade_triplet(seed_colors.magenta_seed_hex, 0.4)?,
        cyan_100: shade_triplet(seed_colors.cyan_seed_hex, 0.0)?,
        cyan_80: shade_triplet(seed_colors.cyan_seed_hex, 0.2)?,
        cyan_60: shade_triplet(seed_colors.cyan_seed_hex, 0.4)?,
    })
}

/// Parses a `#RRGGBB` color and returns RGB channels.
///
/// # Arguments
///
/// - `hex` - Color value in `#RRGGBB` format
///
/// # Errors
///
/// Returns [`ApiError::ValidationError`] if the value is not a valid
/// 6-digit hex color.
fn parse_hex_color(hex: &str) -> Result<(u8, u8, u8), ApiError> {
    let trimmed = hex.trim();

    if trimmed.len() != 7 || !trimmed.starts_with('#') {
        return Err(ApiError::ValidationError(
            "Color values must use 6-digit hex format (for example, #4fa3ff).".to_string(),
        ));
    }

    let red = u8::from_str_radix(&trimmed[1..3], 16).map_err(|_| {
        ApiError::ValidationError(
            "Color values must use 6-digit hex format (for example, #4fa3ff).".to_string(),
        )
    })?;
    let green = u8::from_str_radix(&trimmed[3..5], 16).map_err(|_| {
        ApiError::ValidationError(
            "Color values must use 6-digit hex format (for example, #4fa3ff).".to_string(),
        )
    })?;
    let blue = u8::from_str_radix(&trimmed[5..7], 16).map_err(|_| {
        ApiError::ValidationError(
            "Color values must use 6-digit hex format (for example, #4fa3ff).".to_string(),
        )
    })?;

    Ok((red, green, blue))
}

/// Builds an RGB triplet string from a hex color with optional lightening.
///
/// # Arguments
///
/// - `hex` - Color value in `#RRGGBB` format
/// - `mix_with_white` - Fraction between `0.0` and `1.0` used to lighten color
///
/// # Errors
///
/// Returns [`ApiError::ValidationError`] if `hex` is invalid.
fn shade_triplet(hex: &str, mix_with_white: f32) -> Result<String, ApiError> {
    let (red, green, blue) = parse_hex_color(hex)?;
    let shaded = (
        lighten_channel(red, mix_with_white),
        lighten_channel(green, mix_with_white),
        lighten_channel(blue, mix_with_white),
    );

    Ok(format!("{}, {}, {}", shaded.0, shaded.1, shaded.2))
}

/// Lightens a single RGB channel by mixing toward white.
fn lighten_channel(channel: u8, mix_with_white: f32) -> u8 {
    let mixed = (channel as f32) + ((255.0 - channel as f32) * mix_with_white);
    mixed.round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    //! Unit tests for palette shade generation.
    //!
    //! Covered behavior:
    //! - Valid seed colors produce deterministic 100/80/60 token shades.
    //! - Invalid hex seed values are rejected with validation errors.

    use super::{generate_palette_tokens, PaletteSeedColors};

    #[test]
    // Verifies generated shades follow the expected white-mix algorithm.
    fn generate_palette_tokens_builds_expected_shades() {
        let seeds = PaletteSeedColors {
            background_seed_hex: "#a9b1d6",
            text_seed_hex: "#1a1b26",
            primary_seed_hex: "#9ece6a",
            secondary_seed_hex: "#7aa2f7",
            green_seed_hex: "#336699",
            red_seed_hex: "#e65100",
            yellow_seed_hex: "#f9a825",
            blue_seed_hex: "#1e88e5",
            magenta_seed_hex: "#8e24aa",
            cyan_seed_hex: "#00838f",
        };

        let tokens = generate_palette_tokens(&seeds).expect("token generation should succeed");

        assert_eq!(tokens.green_100, "51, 102, 153");
        assert_eq!(tokens.green_80, "92, 133, 173");
        assert_eq!(tokens.green_60, "133, 163, 194");
        assert_eq!(tokens.background, "169, 177, 214");
        assert_eq!(tokens.text, "26, 27, 38");
        assert_eq!(tokens.primary_100, "158, 206, 106");
        assert_eq!(tokens.secondary_100, "122, 162, 247");
        assert_eq!(tokens.black, "26, 27, 38");
        assert_eq!(tokens.white, "169, 177, 214");
    }

    #[test]
    // Verifies malformed hex values are rejected.
    fn generate_palette_tokens_rejects_invalid_hex_values() {
        let seeds = PaletteSeedColors {
            background_seed_hex: "#a9b1d6",
            text_seed_hex: "#1a1b26",
            primary_seed_hex: "#9ece6a",
            secondary_seed_hex: "#7aa2f7",
            green_seed_hex: "336699",
            red_seed_hex: "#e65100",
            yellow_seed_hex: "#f9a825",
            blue_seed_hex: "#1e88e5",
            magenta_seed_hex: "#8e24aa",
            cyan_seed_hex: "#00838f",
        };

        let result = generate_palette_tokens(&seeds);

        assert!(result.is_err());
    }
}
