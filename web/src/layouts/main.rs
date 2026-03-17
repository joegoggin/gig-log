use leptos::prelude::*;

use crate::components::nav::bar::NavBar;

#[component]
pub fn MainLayout(
    #[prop(optional, into)] class: Option<String>,
    children: Children,
) -> impl IntoView {
    let get_content_class = move || match &class {
        Some(class) => format!("main-layout__content {}", class),
        None => "main-layout__content".to_string(),
    };

    view! {
        <div class="main-layout">
            <NavBar />
            <div class=get_content_class>{children()}</div>
        </div>
    }
}
