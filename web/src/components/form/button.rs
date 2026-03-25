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
    pub fn get_class(&self, optional_class: Option<&str>) -> String {
        match self {
            Self::Primary => {
                ClassNameUtil::add_optional_class("button button--primary", optional_class)
            }
            Self::Secondary => {
                ClassNameUtil::add_optional_class("button button--secondary", optional_class)
            }
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
    let class = variant.get_class(class.as_deref());

    // Event Handlers
    let handle_click = move |ev: MouseEvent| {
        if button_type != ButtonType::Submit {
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
        <button class=class type=move || button_type.to_string() on:click=handle_click>
            {children()}
        </button>
    }
}
