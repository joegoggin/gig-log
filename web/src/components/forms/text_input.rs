//! Styled text input component with validation feedback.

use gig_log_common::models::error::ValidationError;
use leptos::prelude::*;

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
    let error: RwSignal<Option<ValidationError>> = RwSignal::new(None);
    let text_input = RwSignal::new(text_input_normal.clone());
    let has_error = RwSignal::new(false);

    // Variables
    let has_label = label.is_some().clone();

    let input_type = match is_password {
        true => "password",
        false => "text",
    };

    // Effects
    let field = name.clone();

    Effect::new(move || {
        let new_error = errors
            .get()
            .into_iter()
            .find(|validation_error| validation_error.field.as_deref() == Some(field.as_str()));

        match &new_error {
            Some(_) => text_input.set(text_input_error.clone()),
            None => text_input.set(text_input_normal.clone()),
        }

        has_error.set(new_error.is_some());
        error.set(new_error);
    });

    // Event Handlers
    let field = name.clone();

    let on_change = move |e| {
        if has_error.get() {
            let new_errors: Vec<ValidationError> = errors
                .get()
                .into_iter()
                .filter(|error| error.field.as_deref() != Some(field.as_str()))
                .collect();

            has_error.set(false);
            errors.set(new_errors);
        }

        value.set(event_target_value(&e));
    };

    // View
    view! {
        <div class=text_input>
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
            <Show when=move || has_error.get()>
                <p>
                    {move || {
                        error.get().map(|current_error| current_error.message).unwrap_or_default()
                    }}
                </p>
            </Show>
        </div>
    }
}
