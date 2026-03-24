use gig_log_common::models::error::ValidationError;
use leptos::prelude::*;

use crate::{
    components::{
        check_box::CheckBox,
        password_input::PasswordInput,
        select_input::{SelectInput, SelectOption},
        text_area::TextArea,
        text_input::TextInput,
    },
    layouts::auth::AuthLayout,
};

#[component]
pub fn HomePage() -> impl IntoView {
    let error: ValidationError = ValidationError::new(Some("test".to_string()), "This is a test!");
    let errors: RwSignal<Vec<ValidationError>> = RwSignal::new(vec![error]);
    let test = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let checked = RwSignal::new(false);

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
            <TextArea
                name="description"
                placeholder="Description"
                value=description
                errors=errors
                label="Text Area"
            />
            <SelectInput
                options=options
                selected_option=selected_option
                label="Select Input"
                placeholder="Select an option"
            />
            <CheckBox label="This is checkbox" checked=checked />
        </AuthLayout>
    }
}
