//! Root layout wrapper shared by all routes.

use leptos::prelude::*;

use crate::{components::notifications::Notifications, utils::class_name::ClassNameUtil};

const NUMBER_OF_ORBS: u8 = 20;

/// Renders the root page shell and ambient background elements.
///
/// # Arguments
///
/// * `children` — Child route content rendered inside the root layout.
///
/// # Returns
///
/// A Leptos view containing the root layout and notification portal.
#[component]
pub fn RootLayout(children: Children) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new_layout_class_name("root-layout", None);

    let root_layout = class_name.get_root_class();
    let ambient = class_name.get_sub_class("ambient");
    let orb = class_name.get_sub_class("orb");

    view! {
        <main class=root_layout>
            <Notifications />
            <div class=ambient>
                <For
                    each=move || 0..NUMBER_OF_ORBS
                    key=|n| *n
                    children=move |_| {
                        let orb = orb.clone();

                        view! { <span class=orb /> }
                    }
                />
            </div>
            {children()}
        </main>
    }
}
