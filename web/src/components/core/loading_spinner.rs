//! Shared loading indicator component.

use leptos::prelude::*;

/// Renders a default loading spinner with label text.
///
/// # Returns
///
/// A Leptos view containing the loading indicator.
#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="loading">
            <div class="loading__spinner"></div>
            <h3>Loading ...</h3>
        </div>
    }
}
