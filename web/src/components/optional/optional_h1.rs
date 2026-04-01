use leptos::prelude::*;

#[component]
pub fn OptionalH1(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional_no_strip, into)] text: Option<String>,
) -> impl IntoView {
    // Variables
    let has_text = text.is_some();
    let class = class.unwrap_or_default();

    view! {
        <Show when=move || has_text>
            <h1 class=class.clone()>{text.clone().unwrap_or_default()}</h1>
        </Show>
    }
}
