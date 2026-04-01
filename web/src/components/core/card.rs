use leptos::prelude::*;

use crate::{
    components::{OptionalH1, OptionalP},
    utils::class_name::ClassNameUtil,
};

#[component]
pub fn Card(
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] title: Option<String>,
    #[prop(optional, into)] subtitle: Option<String>,
    children: Children,
) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("card", class);
    let card = class_name.get_root_class();

    view! {
        <div class=card>
            <OptionalH1 text=title />
            <OptionalP text=subtitle />
            {children()}
        </div>
    }
}
