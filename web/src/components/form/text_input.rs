use gig_log_common::models::error::ValidationError;
use leptos::prelude::*;

use crate::utils::class_name::ClassNameUtil;

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
    // State
    let error: RwSignal<Option<ValidationError>> = RwSignal::new(None);
    let has_error = RwSignal::new(false);
    let current_class = RwSignal::new(ClassNameUtil::add_optional_class(
        "text-input",
        class.as_deref(),
    ));

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

        let base_class = ClassNameUtil::add_optional_class("text-input", class.as_deref());

        match &new_error {
            Some(_) => current_class.set(ClassNameUtil::add_class(base_class, "text-input--error")),
            None => current_class.set(base_class),
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
        <div class=move || current_class.get()>
            <Show when=move || has_label>
                <label>{label.clone().unwrap_or_default()}</label>
            </Show>
            <input
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
