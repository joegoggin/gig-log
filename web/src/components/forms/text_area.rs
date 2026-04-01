//! Styled textarea component with validation feedback.

use gig_log_common::models::error::ValidationError;
use leptos::prelude::*;

use super::field_validation::FieldValidationState;
use crate::utils::class_name::ClassNameUtil;

/// Renders a textarea input with validation feedback.
///
/// # Arguments
///
/// * `class` — Optional additional CSS class names.
/// * `label` — Optional textarea label text.
/// * `placeholder` — Optional placeholder text.
/// * `rows` — Number of textarea rows.
/// * `name` — Field name used for matching validation errors.
/// * `errors` — Signal containing field validation errors.
/// * `value` — Signal containing the current textarea value.
///
/// # Returns
///
/// A Leptos view containing the textarea field.
#[component]
pub fn TextArea(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, default = 4)] rows: usize,
    #[prop(into)] name: String,
    errors: RwSignal<Vec<ValidationError>>,
    value: RwSignal<String>,
) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new_with_parent("text-input", "text-area", class.clone());

    let text_area_normal = class_name.get_root_class_with_parent();
    let text_area_error = class_name.get_root_class_with_parent_variation("error");

    // State
    let validation =
        FieldValidationState::new(name.clone(), errors, text_area_normal, text_area_error);

    // Variables
    let has_label = label.is_some();

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
            <textarea
                name=name.clone()
                rows=rows
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
