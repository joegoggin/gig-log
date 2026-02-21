//! Appearance-focused repository operations.
//!
//! This module centralizes SQL queries used by appearance customization flows,
//! including custom palette storage and active palette selection persistence.

use sqlx::{Pool, Postgres, types::Json};
use uuid::Uuid;

use crate::models::color_palette::{GeneratedPaletteTokens, UserColorPalette};
use crate::models::user_appearance_preference::UserAppearancePreference;

/// Repository methods for appearance-related persistence.
pub struct AppearanceRepo;

impl AppearanceRepo {
    /// Lists all custom palettes owned by a user, newest first.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn list_custom_palettes_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
    ) -> Result<Vec<UserColorPalette>, sqlx::Error> {
        sqlx::query_as::<_, UserColorPalette>(
            r#"
            SELECT
                id,
                user_id,
                name,
                green_seed_hex,
                red_seed_hex,
                yellow_seed_hex,
                blue_seed_hex,
                magenta_seed_hex,
                cyan_seed_hex,
                generated_tokens,
                generation_version,
                created_at,
                updated_at
            FROM user_color_palettes
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }

    /// Creates a custom palette for a user and returns the inserted record.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `name` - User-provided palette name
    /// - `green_seed_hex` - Green seed color in hex format
    /// - `red_seed_hex` - Red seed color in hex format
    /// - `yellow_seed_hex` - Yellow seed color in hex format
    /// - `blue_seed_hex` - Blue seed color in hex format
    /// - `magenta_seed_hex` - Magenta seed color in hex format
    /// - `cyan_seed_hex` - Cyan seed color in hex format
    /// - `generated_tokens` - Derived palette token map
    /// - `generation_version` - Generator algorithm version
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the insert fails.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_custom_palette_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        name: &str,
        green_seed_hex: &str,
        red_seed_hex: &str,
        yellow_seed_hex: &str,
        blue_seed_hex: &str,
        magenta_seed_hex: &str,
        cyan_seed_hex: &str,
        generated_tokens: &GeneratedPaletteTokens,
        generation_version: i32,
    ) -> Result<UserColorPalette, sqlx::Error> {
        sqlx::query_as::<_, UserColorPalette>(
            r#"
            INSERT INTO user_color_palettes (
                user_id,
                name,
                green_seed_hex,
                red_seed_hex,
                yellow_seed_hex,
                blue_seed_hex,
                magenta_seed_hex,
                cyan_seed_hex,
                generated_tokens,
                generation_version
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING
                id,
                user_id,
                name,
                green_seed_hex,
                red_seed_hex,
                yellow_seed_hex,
                blue_seed_hex,
                magenta_seed_hex,
                cyan_seed_hex,
                generated_tokens,
                generation_version,
                created_at,
                updated_at
            "#,
        )
        .bind(user_id)
        .bind(name)
        .bind(green_seed_hex)
        .bind(red_seed_hex)
        .bind(yellow_seed_hex)
        .bind(blue_seed_hex)
        .bind(magenta_seed_hex)
        .bind(cyan_seed_hex)
        .bind(Json(generated_tokens.clone()))
        .bind(generation_version)
        .fetch_one(pool)
        .await
    }

    /// Finds a custom palette by ID scoped to the owner user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `palette_id` - Custom palette identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_custom_palette_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        palette_id: Uuid,
    ) -> Result<Option<UserColorPalette>, sqlx::Error> {
        sqlx::query_as::<_, UserColorPalette>(
            r#"
            SELECT
                id,
                user_id,
                name,
                green_seed_hex,
                red_seed_hex,
                yellow_seed_hex,
                blue_seed_hex,
                magenta_seed_hex,
                cyan_seed_hex,
                generated_tokens,
                generation_version,
                created_at,
                updated_at
            FROM user_color_palettes
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(palette_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Returns whether a custom palette exists for a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `palette_id` - Custom palette identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn custom_palette_exists_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        palette_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let existing = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id
            FROM user_color_palettes
            WHERE id = $1 AND user_id = $2
            "#,
        )
        .bind(palette_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;

        Ok(existing.is_some())
    }

    /// Loads the active palette preference for a user.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the query fails.
    pub async fn find_active_palette_preference_for_user(
        pool: &Pool<Postgres>,
        user_id: Uuid,
    ) -> Result<Option<UserAppearancePreference>, sqlx::Error> {
        sqlx::query_as::<_, UserAppearancePreference>(
            r#"
            SELECT
                user_id,
                active_palette_type,
                active_preset_palette,
                active_custom_palette_id,
                updated_at
            FROM user_appearance_preferences
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(pool)
        .await
    }

    /// Sets a user's active palette to one of the built-in presets.
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `preset_palette` - Built-in preset palette name
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the upsert fails.
    pub async fn set_active_preset_palette(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        preset_palette: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO user_appearance_preferences (
                user_id,
                active_palette_type,
                active_preset_palette,
                active_custom_palette_id,
                updated_at
            )
            VALUES ($1, 'preset', $2, NULL, NOW())
            ON CONFLICT (user_id)
            DO UPDATE SET
                active_palette_type = 'preset',
                active_preset_palette = EXCLUDED.active_preset_palette,
                active_custom_palette_id = NULL,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(preset_palette)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Sets a user's active palette to a custom palette ID.
    ///
    /// This method does not perform ownership validation; callers should verify
    /// ownership first with [`custom_palette_exists_for_user`](Self::custom_palette_exists_for_user).
    ///
    /// # Arguments
    ///
    /// - `pool` - Database connection pool
    /// - `user_id` - Owner user identifier
    /// - `custom_palette_id` - Custom palette identifier
    ///
    /// # Errors
    ///
    /// Returns `sqlx::Error` if the upsert fails.
    pub async fn set_active_custom_palette(
        pool: &Pool<Postgres>,
        user_id: Uuid,
        custom_palette_id: Uuid,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO user_appearance_preferences (
                user_id,
                active_palette_type,
                active_preset_palette,
                active_custom_palette_id,
                updated_at
            )
            VALUES ($1, 'custom', NULL, $2, NOW())
            ON CONFLICT (user_id)
            DO UPDATE SET
                active_palette_type = 'custom',
                active_preset_palette = NULL,
                active_custom_palette_id = EXCLUDED.active_custom_palette_id,
                updated_at = NOW()
            "#,
        )
        .bind(user_id)
        .bind(custom_palette_id)
        .execute(pool)
        .await?;

        Ok(())
    }
}
