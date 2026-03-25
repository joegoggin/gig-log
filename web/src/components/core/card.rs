use leptos::prelude::*;

use crate::utils::class_name::ClassNameUtil;

#[component]
pub fn Card(#[prop(optional, into)] class: Option<String>, children: Children) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("card", class);
    let card = class_name.get_root_class();

    view! {
        <div class=card>{children()}</div>
    }
}
