//! Layout wrapper for authenticated application pages.

use leptos::prelude::*;

use crate::{components::nav::bar::NavBar, utils::class_name::ClassNameUtil};

/// Renders the primary app layout with navigation and content area.
///
/// # Arguments
///
/// * `class` — Optional additional CSS classes for the content container.
/// * `children` — Child content rendered inside the main layout.
///
/// # Returns
///
/// A Leptos view containing the main application layout.
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
