//! Styled text input component with validation feedback.

use gig_log_common::models::error::ValidationError;
use leptos::prelude::*;

use super::field_validation::FieldValidationState;
use crate::utils::class_name::ClassNameUtil;

/// Renders a text or password input with validation feedback.
///
/// # Arguments
///
/// * `class` — Optional additional CSS class names.
/// * `label` — Optional input label text.
/// * `placeholder` — Optional placeholder text.
/// * `is_password` — Whether to render the input as `password`.
/// * `name` — Field name used for matching validation errors.
/// * `errors` — Signal containing field validation errors.
/// * `value` — Signal containing the current input value.
///
/// # Returns
///
/// A Leptos view containing the text input field.
#[component]
pub fn TextInput(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional)] is_password: bool,
    #[prop(into)] name: String,
    errors: RwSignal<Vec<ValidationError>>,
    value: RwSignal<String>,
) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("text-input", class);
    let text_input_normal = class_name.get_root_class();
    let text_input_error = class_name.get_root_variation("error");

    // State
    let validation =
        FieldValidationState::new(name.clone(), errors, text_input_normal, text_input_error);

    // Variables
    let has_label = label.is_some();

    let input_type = match is_password {
        true => "password",
        false => "text",
    };

    // Event Handlers
    let field = name.clone();

    let on_change = move |e| {
        validation.clear_error(field.as_str(), errors);
        value.set(event_target_value(&e));
    };

    // View
    view! {
        <div class=move || validation.class_name.get()>
            <Show when=move || has_label>
                <label>{label.clone().unwrap_or_default()}</label>
            </Show>
            <input
                name=name.clone()
                type=input_type
                placeholder=placeholder.unwrap_or_default()
                prop:value=value
                on:change=on_change
            />
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
