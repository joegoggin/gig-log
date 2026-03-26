use leptos::prelude::*;

#[component]
pub fn LoadingSpinner() -> impl IntoView {
    view! {
        <div class="loading">
            <div class="loading__spinner"></div>
            <h3>Loading ...</h3>
        </div>
    }
}
