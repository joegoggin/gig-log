//! Styled checkbox form component.

use leptos::prelude::*;

use crate::utils::class_name::ClassNameUtil;

/// Renders a checkbox input bound to a reactive boolean signal.
///
/// # Arguments
///
/// * `class` — Optional additional CSS class names.
/// * `label` — Label text shown next to the checkbox.
/// * `checked` — Signal storing the checkbox checked state.
///
/// # Returns
///
/// A Leptos view containing the checkbox field.
#[component]
pub fn CheckBox(
    #[prop(optional, into)] class: Option<String>,
    #[prop(into)] label: String,
    checked: RwSignal<bool>,
) -> impl IntoView {
    let class = ClassNameUtil::add_optional_class("check-box", class.as_deref());

    let toggle_checked = move |_| {
        checked.update(|checked| *checked = !*checked);
    };

    view! {
        <div class=class>
            <label>
                <input type="checkbox" checked=move || checked.get() on:change=toggle_checked />
                {label}
            </label>
        </div>
    }
}
