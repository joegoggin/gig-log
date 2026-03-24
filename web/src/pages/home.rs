use gig_log_common::models::error::ValidationError;
use leptos::{logging::log, prelude::*};

use crate::{
    components::{
        password_input::PasswordInput,
        select_input::{SelectInput, SelectOption},
        text_input::TextInput,
    },
    layouts::auth::AuthLayout,
};

#[component]
pub fn HomePage() -> impl IntoView {
    let error: ValidationError =
        ValidationError::new(Some("test".to_string()), "This is is a test!");
    let errors: RwSignal<Vec<ValidationError>> = RwSignal::new(vec![error]);
    let test = RwSignal::new(String::new());

    let options: Vec<SelectOption<i8>> = vec![
        SelectOption::new(1, "One"),
        SelectOption::new(2, "Two"),
        SelectOption::new(3, "Three"),
    ];
    let selected_option: RwSignal<Option<SelectOption<i8>>> = RwSignal::new(None);

    view! {
        <AuthLayout>
            <h1>Hello</h1>
            <PasswordInput
                name="test"
                placeholder="Test"
                value=test
                errors=errors
                label="Password Input"
            />
            <TextInput name="test" placeholder="Test" value=test errors=errors label="Text Input" />
            <SelectInput
                options=options
                selected_option=selected_option
                label="Select Input"
                placeholder="Select an option"
            />
        </AuthLayout>
    }
}
