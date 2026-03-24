use gig_log_common::models::error::ValidationError;
use leptos::prelude::*;

use crate::{components::text_input::TextInput, layouts::auth::AuthLayout};

#[component]
pub fn HomePage() -> impl IntoView {
    let error: ValidationError =
        ValidationError::new(Some("test".to_string()), "This is is a test!");
    let errors: RwSignal<Vec<ValidationError>> = RwSignal::new(vec![error]);
    let test = RwSignal::new(String::new());

    view! {
        <AuthLayout>
            <h1>Hello</h1>
            <TextInput name="test" placeholder="Test" value=test errors=errors />
        </AuthLayout>
    }
}
