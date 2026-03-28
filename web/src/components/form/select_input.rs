use std::{collections::HashMap, sync::Arc};

use leptos::prelude::*;

use crate::utils::class_name::ClassNameUtil;

#[derive(Clone, Debug)]
pub struct SelectOption<T> {
    pub value: T,
    pub label: String,
}

impl<T> SelectOption<T> {
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
        }
    }
}

const PLACEHOLDER_VALUE: &str = "__select_input_placeholder__";

#[component]
pub fn SelectInput<T>(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] label: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    options: Vec<SelectOption<T>>,
    selected_option: RwSignal<Option<SelectOption<T>>>,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    let current_class =
        ClassNameUtil::add_optional_class("text-input select-input", class.as_deref());
    let has_label = label.is_some();
    let has_placeholder = placeholder.is_some();
    let placeholder_label = placeholder.unwrap_or_default();

    let labels = options
        .iter()
        .map(|option| option.label.clone())
        .collect::<Vec<_>>();

    let option_by_label = Arc::new(
        options
            .into_iter()
            .map(|option| (option.label.clone(), option))
            .collect::<HashMap<_, _>>(),
    );

    let on_change = move |event| {
        let selected_label = event_target_value(&event);

        if has_placeholder && selected_label == PLACEHOLDER_VALUE {
            selected_option.set(None);
            return;
        }

        selected_option.set(option_by_label.get(&selected_label).cloned());
    };

    view! {
        <div class=current_class>
            <Show when=move || has_label>
                <label>{label.clone().unwrap_or_default()}</label>
            </Show>
            <div class="select-input__field">
                <select
                    prop:value=move || {
                        selected_option
                            .get()
                            .map(|option| option.label)
                            .unwrap_or_else(|| {
                                if has_placeholder {
                                    PLACEHOLDER_VALUE.to_string()
                                } else {
                                    String::new()
                                }
                            })
                    }
                    on:change=on_change
                >
                    <Show when=move || has_placeholder>
                        <option value=PLACEHOLDER_VALUE>
                            {placeholder_label.clone()}
                        </option>
                    </Show>
                    <For
                        each=move || labels.clone()
                        key=|label| label.clone()
                        children=move |label| {
                            view! { <option value=label.clone()>{label.clone()}</option> }
                        }
                    />
                </select>
            </div>
        </div>
    }
}
