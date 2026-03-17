use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::LogOutIcon;

#[derive(Clone)]
pub struct NavItem {
    label: String,
    path: String,
    icon: ViewFn,
}

impl NavItem {
    pub fn new(label: impl Into<String>, path: impl Into<String>, icon: ViewFn) -> Self {
        Self {
            label: label.into(),
            path: path.into(),
            icon,
        }
    }
}

#[component]
pub fn NavMenu(items: Vec<NavItem>, is_active: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="nav-menu">
            <>
                {items
                    .into_iter()
                    .map(|item| {
                        view! {
                            <A href=item.path>
                                <div class="nav-menu__item">
                                    <span class="nav-menu__icon">{item.icon.run()}</span>
                                    <Show when=move || is_active.get()>
                                        <p class="nav-menu__label">{item.label.clone()}</p>
                                    </Show>
                                </div>
                            </A>
                        }
                    })
                    .collect_view()}
            </>
            <div class="nav-menu__item nav-menu__item--log-out">
                <span class="nav-menu__icon">
                    <LogOutIcon />
                </span>
                <Show when=move || is_active.get()>
                    <p class="nav-menu__label">"Log Out"</p>
                </Show>

            </div>
        </div>
    }
}
