use gig_log_common::models::error::ValidationError;
use leptos::prelude::*;

use crate::utils::class_name::ClassNameUtil;

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
    let error: RwSignal<Option<ValidationError>> = RwSignal::new(None);
    let text_area = RwSignal::new(text_area_normal.clone());
    let has_error = RwSignal::new(false);

    // Variables
    let has_label = label.is_some();

    // Effects
    let field = name.clone();

    Effect::new(move || {
        let new_error = errors
            .get()
            .into_iter()
            .find(|validation_error| validation_error.field.as_deref() == Some(field.as_str()));

        match &new_error {
            Some(_) => text_area.set(text_area_error.clone()),
            None => text_area.set(text_area_normal.clone()),
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
        <div class=text_area>
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
