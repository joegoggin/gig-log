//! Password input component with visibility toggle.

use gig_log_common::models::error::ValidationError;
use leptos::prelude::*;

use super::field_validation::FieldValidationState;
use crate::{
    components::{EyeClosedIcon, EyeOpenIcon},
    utils::class_name::ClassNameUtil,
};

/// Renders a password input with validation feedback and visibility toggle.
///
/// # Arguments
///
/// * `class` — Optional additional CSS class names.
/// * `label` — Optional input label text.
/// * `placeholder` — Optional placeholder text.
/// * `name` — Field name used for matching validation errors.
/// * `errors` — Signal containing field validation errors.
/// * `value` — Signal containing the current input value.
///
/// # Returns
///
/// A Leptos view containing the password input field.
#[component]
pub fn PasswordInput(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(into)] name: String,
    errors: RwSignal<Vec<ValidationError>>,
    value: RwSignal<String>,
) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new_with_parent("text-input", "password-input", class.clone());
    let password_input_normal = class_name.get_root_class_with_parent();
    let password_input_error = class_name.get_root_class_with_parent_variation("error");

    // State
    let validation = FieldValidationState::new(
        name.clone(),
        errors,
        password_input_normal,
        password_input_error,
    );
    let is_visible = RwSignal::new(false);

    // Variables
    let has_label = label.is_some();

    // Event Handlers
    let field = name.clone();

    let on_change = move |e| {
        validation.clear_error(field.as_str(), errors);
        value.set(event_target_value(&e));
    };

    let toggle_visibility = move |_| {
        is_visible.update(|visible| *visible = !*visible);
    };

    let input_type = move || match is_visible.get() {
        true => "text",
        false => "password",
    };

    let toggle_button_label = move || match is_visible.get() {
        true => "Hide password",
        false => "Show password",
    };

    // View
    view! {
        <div class=move || validation.class_name.get()>
            <Show when=move || has_label>
                <label>{label.clone().unwrap_or_default()}</label>
            </Show>
            <div class="password-input__field">
                <input
                    name=name.clone()
                    type=input_type
                    placeholder=placeholder.unwrap_or_default()
                    prop:value=value
                    on:change=on_change
                />
                <button
                    type="button"
                    class="password-input__toggle"
                    aria-label=toggle_button_label
                    on:click=toggle_visibility
                >
                    <Show when=move || is_visible.get() fallback=|| view! { <EyeOpenIcon /> }>
                        <EyeClosedIcon />
                    </Show>
                </button>
            </div>
            <Show when=move || validation.has_error.get()>
                <p>
                    {move || {
                        validation
                            .error
                            .get()
                            .map(|current_error| current_error.message)
                            .unwrap_or_default()
                    }}
                </p>
            </Show>
        </div>
    }
}
