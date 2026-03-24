use leptos::prelude::*;

use crate::{components::nav::bar::NavBar, utils::class_name::ClassNameUtil};

#[component]
pub fn MainLayout(
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let content_class = ClassNameUtil::add_optional_class("main-layout__content", class.as_deref());

    view! {
        <div class="main-layout">
            <NavBar />
            <div class=content_class>{children()}</div>
        </div>
    }
}
