//! User appearance preference model for persisted palette selection.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Persisted active palette selection for a user.
#[derive(Debug, Serialize, Deserialize, FromRow)]
#[allow(dead_code)]
pub struct UserAppearancePreference {
    /// User identifier that owns this preference record.
    pub user_id: Uuid,
    /// Active palette source type (`preset` or `custom`).
    pub active_palette_type: String,
    /// Active preset palette name when `active_palette_type` is `preset`.
    pub active_preset_palette: Option<String>,
    /// Active custom palette ID when `active_palette_type` is `custom`.
    pub active_custom_palette_id: Option<Uuid>,
    /// Timestamp when the active palette selection was last updated.
    pub updated_at: DateTime<Utc>,
}
