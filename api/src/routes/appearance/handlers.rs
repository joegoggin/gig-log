//! HTTP handler functions for appearance endpoints.
//!
//! This module contains handlers for loading appearance settings, creating
//! user-defined custom color palettes, and updating active palette selection.

use actix_web::{HttpResponse, get, post, put, web};
use serde_json::Value;

use crate::auth::middleware::AuthenticatedUser;
use crate::core::app_state::AppState;
use crate::core::error::{ApiError, ApiResult};
use crate::extractors::ValidatedJson;
use crate::models::color_palette::{GeneratedPaletteTokens, UserColorPalette};
use crate::models::user_appearance_preference::UserAppearancePreference;
use crate::repository::appearance::AppearanceRepo;
use crate::services::palette_generation::{
    PALETTE_GENERATION_VERSION, PaletteSeedColors, generate_palette_tokens,
};

use super::payloads::{
    ActivePaletteSelectionResponse, CreateCustomPaletteRequest, CreateCustomPaletteResponse,
    CustomPaletteResponse, GetAppearanceResponse, SetActivePaletteRequest,
    SetActivePaletteResponse,
};

const PRESET_PALETTES: [&str; 3] = ["default", "sunset", "forest"];

/// Loads appearance settings for the authenticated user.
///
/// Returns the active palette selection and all custom palettes owned by the
/// current user.
///
/// # Route
///
/// `GET /appearance`
///
/// # Response Body ([`GetAppearanceResponse`])
///
/// - `active_palette` - Current palette selection (`preset` or `custom`)
/// - `custom_palettes` - User-owned custom palettes
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `DatabaseError` - If appearance data retrieval fails
#[get("/appearance")]
pub async fn get_appearance(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
) -> ApiResult<HttpResponse> {
    let palettes =
        AppearanceRepo::list_custom_palettes_for_user(&state.pool, auth_user.user_id).await?;
    let custom_palettes = palettes
        .into_iter()
        .map(map_custom_palette)
        .collect::<ApiResult<Vec<_>>>()?;

    let preference =
        AppearanceRepo::find_active_palette_preference_for_user(&state.pool, auth_user.user_id)
            .await?;
    let active_palette = resolve_active_palette(preference, &custom_palettes);

    Ok(HttpResponse::Ok().json(GetAppearanceResponse {
        active_palette,
        custom_palettes,
    }))
}

/// Creates a custom palette for the authenticated user.
///
/// Accepts six base accent colors and generates a full 100/80/60 token set.
/// The newly created custom palette becomes the active palette.
///
/// # Route
///
/// `POST /appearance/palettes`
///
/// # Request Body ([`CreateCustomPaletteRequest`])
///
/// - `name` - User-defined palette name
/// - `background_seed_hex` - Background base color in `#RRGGBB`
/// - `text_seed_hex` - Text base color in `#RRGGBB`
/// - `primary_seed_hex` - Primary action base color in `#RRGGBB`
/// - `secondary_seed_hex` - Secondary action base color in `#RRGGBB`
/// - `green_seed_hex` - Green base accent in `#RRGGBB`
/// - `red_seed_hex` - Red base accent in `#RRGGBB`
/// - `yellow_seed_hex` - Yellow base accent in `#RRGGBB`
/// - `blue_seed_hex` - Blue base accent in `#RRGGBB`
/// - `magenta_seed_hex` - Magenta base accent in `#RRGGBB`
/// - `cyan_seed_hex` - Cyan base accent in `#RRGGBB`
///
/// # Response Body ([`CreateCustomPaletteResponse`])
///
/// - `message` - Success message
/// - `palette` - Newly created custom palette
/// - `active_palette` - Updated active palette selection (`custom`)
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `ValidationError` - If payload validation fails or palette name already exists
/// - `DatabaseError` - If palette creation fails
#[post("/appearance/palettes")]
pub async fn create_custom_palette(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    body: ValidatedJson<CreateCustomPaletteRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();
    let normalized_name = body.name.trim();

    if normalized_name.is_empty() {
        return Err(ApiError::ValidationError(
            "Palette name is required".to_string(),
        ));
    }

    let background_seed_hex = body.background_seed_hex.trim().to_lowercase();
    let text_seed_hex = body.text_seed_hex.trim().to_lowercase();
    let primary_seed_hex = body.primary_seed_hex.trim().to_lowercase();
    let secondary_seed_hex = body.secondary_seed_hex.trim().to_lowercase();
    let green_seed_hex = body.green_seed_hex.trim().to_lowercase();
    let red_seed_hex = body.red_seed_hex.trim().to_lowercase();
    let yellow_seed_hex = body.yellow_seed_hex.trim().to_lowercase();
    let blue_seed_hex = body.blue_seed_hex.trim().to_lowercase();
    let magenta_seed_hex = body.magenta_seed_hex.trim().to_lowercase();
    let cyan_seed_hex = body.cyan_seed_hex.trim().to_lowercase();

    let normalized_seeds = PaletteSeedColors {
        background_seed_hex: &background_seed_hex,
        text_seed_hex: &text_seed_hex,
        primary_seed_hex: &primary_seed_hex,
        secondary_seed_hex: &secondary_seed_hex,
        green_seed_hex: &green_seed_hex,
        red_seed_hex: &red_seed_hex,
        yellow_seed_hex: &yellow_seed_hex,
        blue_seed_hex: &blue_seed_hex,
        magenta_seed_hex: &magenta_seed_hex,
        cyan_seed_hex: &cyan_seed_hex,
    };
    let generated_tokens = generate_palette_tokens(&normalized_seeds)?;

    let created_palette = AppearanceRepo::create_custom_palette_for_user(
        &state.pool,
        auth_user.user_id,
        normalized_name,
        &background_seed_hex,
        &text_seed_hex,
        &primary_seed_hex,
        &secondary_seed_hex,
        &green_seed_hex,
        &red_seed_hex,
        &yellow_seed_hex,
        &blue_seed_hex,
        &magenta_seed_hex,
        &cyan_seed_hex,
        &generated_tokens,
        PALETTE_GENERATION_VERSION,
    )
    .await
    .map_err(map_create_palette_error)?;

    AppearanceRepo::set_active_custom_palette(&state.pool, auth_user.user_id, created_palette.id)
        .await?;

    let palette = map_custom_palette(created_palette)?;
    let active_palette = ActivePaletteSelectionResponse {
        palette_type: "custom".to_string(),
        preset_palette: None,
        custom_palette_id: Some(palette.id),
    };

    Ok(HttpResponse::Created().json(CreateCustomPaletteResponse {
        message: "Custom palette created successfully.".to_string(),
        palette,
        active_palette,
    }))
}

/// Updates the authenticated user's active palette selection.
///
/// Allows switching between built-in preset palettes and user-owned custom
/// palettes.
///
/// # Route
///
/// `PUT /appearance/active-palette`
///
/// # Request Body ([`SetActivePaletteRequest`])
///
/// - `palette_type` - Selection source (`preset` or `custom`)
/// - `preset_palette` - Required when `palette_type` is `preset`
/// - `custom_palette_id` - Required when `palette_type` is `custom`
///
/// # Response Body ([`SetActivePaletteResponse`])
///
/// - `message` - Success message
/// - `active_palette` - Updated active palette selection
///
/// # Errors
///
/// - `Unauthorized` - If the request does not include a valid access token
/// - `ValidationError` - If request fields are inconsistent or invalid
/// - `NotFound` - If a custom palette ID is not owned by the user
/// - `DatabaseError` - If the update fails
#[put("/appearance/active-palette")]
pub async fn set_active_palette(
    state: web::Data<AppState>,
    auth_user: AuthenticatedUser,
    body: ValidatedJson<SetActivePaletteRequest>,
) -> ApiResult<HttpResponse> {
    let body = body.into_inner();
    let palette_type = body.palette_type.trim().to_lowercase();

    let active_palette = match palette_type.as_str() {
        "preset" => {
            let preset_palette = body
                .preset_palette
                .as_deref()
                .map(str::trim)
                .ok_or(ApiError::ValidationError(
                    "Preset palette is required when palette_type is preset".to_string(),
                ))?
                .to_lowercase();

            if !is_preset_palette(&preset_palette) {
                return Err(ApiError::ValidationError(
                    "Preset palette must be one of: default, sunset, forest".to_string(),
                ));
            }

            AppearanceRepo::set_active_preset_palette(
                &state.pool,
                auth_user.user_id,
                &preset_palette,
            )
            .await?;

            ActivePaletteSelectionResponse {
                palette_type: "preset".to_string(),
                preset_palette: Some(preset_palette),
                custom_palette_id: None,
            }
        }
        "custom" => {
            let custom_palette_id = body.custom_palette_id.ok_or(ApiError::ValidationError(
                "custom_palette_id is required when palette_type is custom".to_string(),
            ))?;

            let exists = AppearanceRepo::custom_palette_exists_for_user(
                &state.pool,
                auth_user.user_id,
                custom_palette_id,
            )
            .await?;

            if !exists {
                return Err(ApiError::NotFound("Custom palette not found".to_string()));
            }

            AppearanceRepo::set_active_custom_palette(
                &state.pool,
                auth_user.user_id,
                custom_palette_id,
            )
            .await?;

            ActivePaletteSelectionResponse {
                palette_type: "custom".to_string(),
                preset_palette: None,
                custom_palette_id: Some(custom_palette_id),
            }
        }
        _ => {
            return Err(ApiError::ValidationError(
                "palette_type must be either preset or custom".to_string(),
            ));
        }
    };

    Ok(HttpResponse::Ok().json(SetActivePaletteResponse {
        message: "Active palette updated successfully.".to_string(),
        active_palette,
    }))
}

fn is_preset_palette(value: &str) -> bool {
    PRESET_PALETTES.contains(&value)
}

fn resolve_active_palette(
    preference: Option<UserAppearancePreference>,
    custom_palettes: &[CustomPaletteResponse],
) -> ActivePaletteSelectionResponse {
    match preference {
        Some(preference) if preference.active_palette_type == "preset" => {
            if let Some(preset_palette) = preference.active_preset_palette {
                let normalized = preset_palette.to_lowercase();

                if is_preset_palette(&normalized) {
                    return ActivePaletteSelectionResponse {
                        palette_type: "preset".to_string(),
                        preset_palette: Some(normalized),
                        custom_palette_id: None,
                    };
                }
            }
        }
        Some(preference) if preference.active_palette_type == "custom" => {
            if let Some(custom_palette_id) = preference.active_custom_palette_id {
                let exists = custom_palettes
                    .iter()
                    .any(|palette| palette.id == custom_palette_id);

                if exists {
                    return ActivePaletteSelectionResponse {
                        palette_type: "custom".to_string(),
                        preset_palette: None,
                        custom_palette_id: Some(custom_palette_id),
                    };
                }
            }
        }
        _ => {}
    }

    ActivePaletteSelectionResponse {
        palette_type: "preset".to_string(),
        preset_palette: Some("default".to_string()),
        custom_palette_id: None,
    }
}

fn parse_generated_tokens(value: Value) -> ApiResult<GeneratedPaletteTokens> {
    serde_json::from_value::<GeneratedPaletteTokens>(value).map_err(|error| {
        ApiError::InternalError(format!(
            "Stored palette tokens are invalid and could not be decoded: {error}"
        ))
    })
}

fn map_custom_palette(palette: UserColorPalette) -> ApiResult<CustomPaletteResponse> {
    let generated_tokens = parse_generated_tokens(palette.generated_tokens)?;

    Ok(CustomPaletteResponse {
        id: palette.id,
        name: palette.name,
        background_seed_hex: palette.background_seed_hex,
        text_seed_hex: palette.text_seed_hex,
        primary_seed_hex: palette.primary_seed_hex,
        secondary_seed_hex: palette.secondary_seed_hex,
        green_seed_hex: palette.green_seed_hex,
        red_seed_hex: palette.red_seed_hex,
        yellow_seed_hex: palette.yellow_seed_hex,
        blue_seed_hex: palette.blue_seed_hex,
        magenta_seed_hex: palette.magenta_seed_hex,
        cyan_seed_hex: palette.cyan_seed_hex,
        generated_tokens,
        generation_version: palette.generation_version,
        created_at: palette.created_at,
        updated_at: palette.updated_at,
    })
}

fn map_create_palette_error(error: sqlx::Error) -> ApiError {
    if let sqlx::Error::Database(database_error) = &error {
        if database_error.constraint() == Some("idx_user_color_palettes_user_lower_name") {
            return ApiError::ValidationError(
                "You already have a custom palette with this name.".to_string(),
            );
        }
    }

    ApiError::from(error)
}

#[cfg(test)]
mod tests {
    //! Handler-level tests for appearance routes.
    //!
    //! Covered behavior:
    //! - Auth guard rejects unauthenticated appearance requests.
    //! - Request validation rejects invalid color-seed payloads.
    //! - Active-palette updates reject inconsistent request bodies.

    use actix_web::{App, http::StatusCode, test, web};
    use async_trait::async_trait;
    use serde_json::json;
    use sqlx::postgres::PgPoolOptions;
    use uuid::Uuid;

    use crate::auth::jwt::create_access_token;
    use crate::core::app_state::AppState;
    use crate::core::env::Env;
    use crate::core::error::ApiError;
    use crate::services::email::EmailSender;

    use super::{create_custom_palette, get_appearance, set_active_palette};
    use std::sync::Arc;

    #[derive(Debug, Default)]
    struct NoopEmailSender;

    #[async_trait]
    impl EmailSender for NoopEmailSender {
        async fn send_confirmation_email(
            &self,
            _to_email: &str,
            _first_name: &str,
            _code: &str,
        ) -> Result<(), ApiError> {
            Ok(())
        }

        async fn send_password_reset_email(
            &self,
            _to_email: &str,
            _first_name: &str,
            _code: &str,
        ) -> Result<(), ApiError> {
            Ok(())
        }

        async fn send_email_change_email(
            &self,
            _to_email: &str,
            _first_name: &str,
            _code: &str,
        ) -> Result<(), ApiError> {
            Ok(())
        }
    }

    fn test_env() -> Env {
        Env {
            app_env: "test".to_string(),
            docker_compose_auto_start_enabled: false,
            auto_apply_migrations_enabled: false,
            database_url: "postgres://localhost/test".to_string(),
            cors_allowed_origin: "http://localhost:3000".to_string(),
            port: 0,
            jwt_secret: "appearance-handler-test-secret".to_string(),
            jwt_access_token_expiry_seconds: 900,
            jwt_refresh_token_expiry_seconds: 604_800,
            resend_api_key: "test-resend-key".to_string(),
            resend_from_email: "test@giglog.dev".to_string(),
            auth_code_expiry_seconds: 600,
            cookie_domain: Some("localhost".to_string()),
            cookie_secure: false,
            log_level: "info".to_string(),
            log_http_body_enabled: false,
            log_http_max_body_bytes: 16_384,
        }
    }

    fn test_state() -> AppState {
        let pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_lazy("postgres://postgres:postgres@localhost:5432/postgres")
            .expect("lazy pool should be created for handler tests");

        AppState::with_email_sender(pool, test_env(), Arc::new(NoopEmailSender))
    }

    #[actix_web::test]
    // Verifies unauthenticated appearance requests are rejected by auth extraction.
    async fn get_appearance_without_access_cookie_returns_unauthorized() {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(test_state()))
                .service(get_appearance),
        )
        .await;

        let request = test::TestRequest::get().uri("/appearance").to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    // Verifies invalid hex seed payloads are rejected by request validation.
    async fn create_custom_palette_with_invalid_seed_hex_returns_bad_request() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "appearance-handler@test.dev",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("access token should be created for handler test");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(create_custom_palette),
        )
        .await;

        let request = test::TestRequest::post()
            .uri("/appearance/palettes")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .set_json(json!({
                "name": "Ocean",
                "background_seed_hex": "#a9b1d6",
                "text_seed_hex": "#1a1b26",
                "primary_seed_hex": "#9ece6a",
                "secondary_seed_hex": "#7aa2f7",
                "green_seed_hex": "not-a-hex",
                "red_seed_hex": "#e27d7c",
                "yellow_seed_hex": "#d0a761",
                "blue_seed_hex": "#5c93cd",
                "magenta_seed_hex": "#a082ce",
                "cyan_seed_hex": "#59b7aa"
            }))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body: serde_json::Value = test::read_body_json(response).await;
        assert!(
            body.get("errors")
                .and_then(|value| value.as_array())
                .is_some()
        );
    }

    #[actix_web::test]
    // Verifies active-palette requests enforce required custom palette IDs.
    async fn set_active_palette_custom_without_custom_palette_id_returns_bad_request() {
        let state = test_state();
        let access_token = create_access_token(
            Uuid::new_v4(),
            "appearance-handler@test.dev",
            &state.env.jwt_secret,
            state.env.jwt_access_token_expiry_seconds,
        )
        .expect("access token should be created for handler test");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .service(set_active_palette),
        )
        .await;

        let request = test::TestRequest::put()
            .uri("/appearance/active-palette")
            .insert_header(("Cookie", format!("access_token={access_token}")))
            .set_json(json!({
                "palette_type": "custom"
            }))
            .to_request();
        let response = test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body: serde_json::Value = test::read_body_json(response).await;
        assert_eq!(
            body.get("error")
                .and_then(|error| error.get("code"))
                .and_then(|code| code.as_str()),
            Some("VALIDATION_ERROR")
        );
    }
}
