//! Appearance handlers for persisted palette preferences and custom palettes.
//!
//! This module provides HTTP handlers for loading appearance settings,
//! creating and updating user-defined color palettes, and selecting active
//! preset/custom palettes for authenticated users.
//!
//! # Module Structure
//!
//! - [`handlers`] - HTTP handler functions for appearance endpoints
//! - [`payloads`] - Request and response data structures

pub mod handlers;
pub mod payloads;

// Re-export handlers at module level for easy route registration.
pub use handlers::{
    create_custom_palette, get_appearance, set_active_palette, update_custom_palette,
};

// Re-export payload types that are used by other modules.
pub use payloads::{
    CreateCustomPaletteRequest, SetActivePaletteRequest, UpdateCustomPaletteRequest,
};
