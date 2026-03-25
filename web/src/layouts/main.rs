use leptos::prelude::*;

use crate::{components::nav::bar::NavBar, utils::class_name::ClassNameUtil};

#[component]
pub fn MainLayout(
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new_layout_class_name("main-layout", class);

    let main_layout = class_name.get_root_class();
    let content = class_name.get_content_class();

    view! {
        <div class=main_layout>
            <NavBar />
            <div class=content>{children()}</div>
        </div>
    }
}
