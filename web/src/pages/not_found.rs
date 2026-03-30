//! Page component for `NotFoundPage`.

use leptos::prelude::*;

/// Renders the `NotFoundPage` component.
///
/// # Returns
///
/// A Leptos view for the `NotFoundPage` UI.
#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <div>
            <h1>"404 - Page Not Found"</h1>
        </div>
    }
}
