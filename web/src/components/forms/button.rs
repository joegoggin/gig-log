use leptos::{ev::MouseEvent, prelude::*};
use leptos_router::hooks::use_navigate;

use crate::utils::class_name::ClassNameUtil;

#[derive(PartialEq, Clone, Copy)]
pub enum ButtonType {
    Submit,
    Button,
    Reset,
}

impl ToString for ButtonType {
    fn to_string(&self) -> String {
        match self {
            Self::Submit => "submit".to_string(),
            Self::Button => "button".to_string(),
            Self::Reset => "reset".to_string(),
        }
    }
}

pub enum ButtonVariant {
    Primary,
    Secondary,
}

impl ButtonVariant {
    pub fn get_class(&self, class: Option<String>) -> String {
        let class_name = ClassNameUtil::new("button", class);

        match self {
            Self::Primary => class_name.get_root_variation("primary"),
            Self::Secondary => class_name.get_root_variation("secondary"),
        }
    }
}

#[component]
pub fn Button(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] on_click: Option<Callback<MouseEvent>>,
    #[prop(optional, into)] href: Option<String>,
    #[prop(optional, default = ButtonVariant::Primary )] variant: ButtonVariant,
    #[prop(optional, default = ButtonType::Button)] button_type: ButtonType,
    children: Children,
) -> impl IntoView {
    // Hooks
    let navigate = use_navigate();

    // Variables
    let button = variant.get_class(class);

    // Event Handlers
    let handle_click = move |ev: MouseEvent| {
        if href.is_some() || button_type == ButtonType::Button {
            ev.prevent_default();
        }

        if let Some(on_click) = on_click.as_ref() {
            on_click.run(ev)
        }

        if let Some(href) = href.as_ref() {
            navigate(&href, Default::default())
        }
    };

    view! {
        <button class=button type=move || button_type.to_string() on:click=handle_click>
            {children()}
        </button>
    }
}
