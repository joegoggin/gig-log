use leptos::prelude::*;

#[component]
pub fn OptionalP(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional_no_strip, into)] text: Option<String>,
) -> impl IntoView {
    // Variables
    let has_text = text.is_some();
    let class = class.unwrap_or_default();

    view! {
        <Show when=move || has_text>
            <p class=class.clone()>{text.clone().unwrap_or_default()}</p>
        </Show>
    }
}
