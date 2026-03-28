use leptos::prelude::*;

use crate::components::notifications::Notifications;

const NUMBER_OF_ORBS: u8 = 20;

#[component]
pub fn RootLayout(children: Children) -> impl IntoView {
    view! {
        <main class="root-layout">
            <Notifications />
            <div class="root-layout__ambient">
                <For
                    each=move || 0..NUMBER_OF_ORBS
                    key=|n| *n
                    children=|_| view! { <span class="root-layout__orb" /> }
                />
            </div>
            {children()}
        </main>
    }
}
