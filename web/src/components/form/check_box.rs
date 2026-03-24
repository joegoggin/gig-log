use leptos::prelude::*;

use crate::utils::class_name::ClassNameUtil;

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
            <input type="checkbox" checked=move || checked.get() on:change=toggle_checked />
            <label>{label}</label>
        </div>
    }
}
