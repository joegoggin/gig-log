use leptos::prelude::*;

use crate::utils::class_name::ClassNameUtil;

#[component]
pub fn AuthLayout(
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let content_class = ClassNameUtil::add_optional_class("auth-layout__content", class.as_deref());

    view! {
        <div class="auth-layout">
            <div class=content_class>{children()}</div>
        </div>
    }
}
