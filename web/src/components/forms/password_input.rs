use gig_log_common::models::error::ValidationError;
use leptos::prelude::*;

use crate::{
    components::{EyeClosedIcon, EyeOpenIcon},
    utils::class_name::ClassNameUtil,
};

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
    let error: RwSignal<Option<ValidationError>> = RwSignal::new(None);
    let password_input = RwSignal::new(password_input_normal.clone());
    let has_error = RwSignal::new(false);
    let is_visible = RwSignal::new(false);

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
            Some(_) => password_input.set(password_input_error.clone()),
            None => password_input.set(password_input_normal.clone()),
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
        <div class=move || password_input.get()>
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
