//! Navigation menu items and rendering component.

use leptos::{prelude::*, reactive::spawn_local};
use leptos_router::components::A;

use crate::{components::LogOutIcon, contexts::use_auth};

/// Represents one item rendered in the navigation menu.
#[derive(Clone)]
pub struct NavItem {
    /// Stores the item label displayed to the user.
    label: String,
    /// Stores the destination path for the item.
    path: String,
    /// Stores the icon renderer for the item.
    icon: ViewFn,
}

impl NavItem {
    /// Creates a new navigation item.
    ///
    /// # Arguments
    ///
    /// * `label` — Display label for the item.
    /// * `path` — Route path for the item link.
    /// * `icon` — Icon renderer for the item.
    ///
    /// # Returns
    ///
    /// An initialized [`NavItem`].
    pub fn new(label: impl Into<String>, path: impl Into<String>, icon: ViewFn) -> Self {
        Self {
            label: label.into(),
            path: path.into(),
            icon,
        }
    }
}

/// Renders the navigation menu and logout action.
///
/// # Arguments
///
/// * `items` — Navigation items rendered as links.
/// * `is_active` — Signal indicating whether the nav is hovered/active.
/// * `is_mobile_menu_open` — Signal indicating mobile menu open state.
///
/// # Returns
///
/// A Leptos view containing the menu links and logout button.
#[component]
pub fn NavMenu(
    items: Vec<NavItem>,
    is_active: RwSignal<bool>,
    is_mobile_menu_open: RwSignal<bool>,
) -> impl IntoView {
    let auth = use_auth();
    let when_show_text = move || is_active.get() || is_mobile_menu_open.get();

    let on_log_out = move |_| {
        let auth = auth.clone();
        is_mobile_menu_open.set(false);

        spawn_local(async move {
            let _ = auth.logout().await;
        });
    };

    view! {
        <div class="nav-menu">
            <>
                {items
                    .into_iter()
                    .map(|item| {
                        let label_for_aria = item.label.clone();

                        view! {
                            <A
                                href=item.path
                                attr:aria-label=label_for_aria
                                on:click=move |_| is_mobile_menu_open.set(false)
                            >
                                <div class="nav-menu__item">
                                    <span class="nav-menu__icon">{item.icon.run()}</span>
                                    <Show when=when_show_text>
                                        <p class="nav-menu__label">{item.label.clone()}</p>
                                    </Show>
                                </div>
                            </A>
                        }
                    })
                    .collect_view()}
            </>
            <button
                type="button"
                class="nav-menu__item nav-menu__item--log-out"
                aria-label="Log Out"
                on:click=on_log_out
            >
                <span class="nav-menu__icon">
                    <LogOutIcon />
                </span>
                <Show when=when_show_text>
                    <p class="nav-menu__label">"Log Out"</p>
                </Show>
            </button>
        </div>
    }
}
