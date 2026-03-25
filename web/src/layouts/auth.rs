use leptos::prelude::*;

use crate::utils::class_name::ClassNameUtil;

#[component]
pub fn AuthLayout(
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new_layout_class_name("auth-layout", class);

    let auth_layout = class_name.get_root_class();
    let content = class_name.get_content_class();

    view! {
        <div class=auth_layout>
            <div class=content>{children()}</div>
        </div>
    }
}
