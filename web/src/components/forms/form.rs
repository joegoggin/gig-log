//! Reusable form wrapper with submit handling and loading overlay.

use leptos::{ev::SubmitEvent, prelude::*};

use crate::utils::class_name::ClassNameUtil;

/// Renders a reusable form wrapper with centralized loading behavior.
///
/// When `is_loading` is `true`, submissions are ignored and the component
/// shows a centered loading spinner overlay while preserving form dimensions.
///
/// # Arguments
///
/// * `class` — Optional additional CSS class names.
/// * `on_submit` — Callback invoked after preventing the default form submit.
/// * `is_loading` — Signal indicating whether the form is currently submitting.
/// * `children` — Form field content rendered inside the wrapper.
///
/// # Returns
///
/// A Leptos view containing the form and loading overlay.
#[component]
pub fn Form(
    #[prop(optional, into)] class: Option<String>,
    #[prop(into)] on_submit: Callback<SubmitEvent>,
    is_loading: RwSignal<bool>,
    children: Children,
) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("form", class);
    let form = class_name.get_root_class();
    let form_loading = class_name.get_root_variation("loading");

    let form_class = move || {
        if is_loading.get() {
            form_loading.clone()
        } else {
            form.clone()
        }
    };

    // Event Handlers
    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        if is_loading.get_untracked() {
            return;
        }

        on_submit.run(ev);
    };

    view! {
        <form class=form_class on:submit=handle_submit>
            {children()}
            <Show when=move || is_loading.get()>
                <div class="form__loading-overlay" aria-hidden="true">
                    <div class="form__loading-spinner"></div>
                </div>
            </Show>
        </form>
    }
}
