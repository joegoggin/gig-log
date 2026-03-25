use leptos::prelude::*;

use crate::utils::class_name::ClassNameUtil;

#[component]
pub fn Card(#[prop(optional, into)] class: Option<String>, children: Children) -> impl IntoView {
    let class = ClassNameUtil::add_optional_class("card", class.as_deref());

    view! {
        <div class=class>{children()}</div>
    }
}
