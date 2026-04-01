//! Reusable auth-page wrapper that composes shared layout and form shells.

use leptos::{ev::SubmitEvent, prelude::*};

use crate::{
    components::{Card, Form},
    layouts::auth::AuthLayout,
};

/// Renders a standardized auth page form card.
///
/// # Arguments
///
/// * `title` — Heading text shown at the top of the card.
/// * `subtitle` — Supporting description shown below the title.
/// * `is_loading` — Signal indicating whether the form is currently submitting.
/// * `on_submit` — Callback invoked when the form is submitted.
/// * `children` — Field content rendered inside the form.
///
/// # Returns
///
/// A Leptos view containing the auth layout, card, and form wrapper.
#[component]
pub fn AuthFormCard(
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    is_loading: RwSignal<bool>,
    #[prop(into)] on_submit: Callback<SubmitEvent>,
    children: Children,
) -> impl IntoView {
    view! {
        <AuthLayout>
            <Card title=title subtitle=subtitle>
                <Form on_submit=on_submit is_loading=is_loading>
                    {children()}
                </Form>
            </Card>
        </AuthLayout>
    }
}
