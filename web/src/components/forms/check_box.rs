use leptos::prelude::*;

use crate::utils::class_name::ClassNameUtil;

#[component]
pub fn CheckBox(
    #[prop(optional, into)] class: Option<String>,
    #[prop(into)] label: String,
    checked: RwSignal<bool>,
) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("check-box", class);
    let check_box = class_name.get_root_class();

    let toggle_checked = move |_| {
        checked.update(|checked| *checked = !*checked);
    };

    view! {
        <div class=check_box>
            <label>
                <input type="checkbox" checked=move || checked.get() on:change=toggle_checked />
                {label}
            </label>
        </div>
    }
}
