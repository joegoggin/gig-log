//! Shared helpers used across authentication page components.

/// Provides a composed auth layout + card + form wrapper component.
pub mod auth_form_card;
/// Provides shared auth form state and submit lifecycle helpers.
pub mod form_state;

pub use auth_form_card::AuthFormCard;
pub use form_state::{submit_auth_form, use_auth_form};
