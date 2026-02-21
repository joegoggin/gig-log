//! Hex color validation helpers for appearance payloads.
//!
//! This module provides a reusable validator for enforcing 6-digit hex color
//! values used by custom palette creation requests.

/// Validates that a color value uses 6-digit hex format (`#RRGGBB`).
///
/// Used with `#[validate(custom(function = "validate_hex_color"))]` on
/// appearance payload fields.
///
/// See [`create_custom_palette`](crate::routes::appearance::handlers::create_custom_palette)
/// for the handler that uses this validation.
pub fn validate_hex_color(value: &str) -> Result<(), validator::ValidationError> {
    let trimmed = value.trim();

    let is_valid = trimmed.len() == 7
        && trimmed.starts_with('#')
        && trimmed
            .chars()
            .skip(1)
            .all(|character| character.is_ascii_hexdigit());

    if is_valid {
        return Ok(());
    }

    let mut error = validator::ValidationError::new("invalid_hex_color");
    error.message = Some("Color must use 6-digit hex format (for example, #4fa3ff).".into());
    Err(error)
}

#[cfg(test)]
mod tests {
    //! Unit tests for hex color validator behavior.
    //!
    //! Covered behavior:
    //! - Valid `#RRGGBB` values are accepted.
    //! - Missing hash, wrong length, or non-hex characters are rejected.

    use super::validate_hex_color;

    #[test]
    // Verifies valid 6-digit hex values pass validation.
    fn validate_hex_color_accepts_valid_values() {
        assert!(validate_hex_color("#4fa3ff").is_ok());
        assert!(validate_hex_color("#ABC123").is_ok());
    }

    #[test]
    // Verifies invalid hex formats are rejected.
    fn validate_hex_color_rejects_invalid_values() {
        assert!(validate_hex_color("4fa3ff").is_err());
        assert!(validate_hex_color("#fff").is_err());
        assert!(validate_hex_color("#zzzzzz").is_err());
    }
}
