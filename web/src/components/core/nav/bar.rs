//! Primary navigation bar component.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::{
    components::{
        CompanyIcon, HamburgerIcon, HomeIcon, JobIcon, LogoIcon, PaymentIcon,
        nav::menu::{NavItem, NavMenu},
        settings::SettingsIcon,
    },
    contexts::use_mobile,
};

/// Renders the primary sidebar navigation bar.
///
/// # Returns
///
/// A Leptos view containing the navigation shell and menu.
#[component]
pub fn NavBar() -> impl IntoView {
    let mobile = use_mobile();

    let is_active = RwSignal::new(false);
    let is_mobile_menu_open = RwSignal::new(false);

    let items: Vec<NavItem> = vec![
        NavItem::new("Dashboard", "/dashboard", HomeIcon.into()),
        NavItem::new("Companies", "/companies", CompanyIcon.into()),
        NavItem::new("Jobs", "/jobs", JobIcon.into()),
        NavItem::new("Payments", "/payments", PaymentIcon.into()),
        NavItem::new("Settings", "/settings", SettingsIcon.into()),
    ];

    let get_class = move || match is_mobile_menu_open.get() {
        true => "nav-bar nav-bar--menu-open",
        false => "nav-bar",
    };

    let toggle_mobile_menu = move |_| is_mobile_menu_open.update(|is_open| *is_open = !*is_open);

    let close_menu = move |_| is_mobile_menu_open.set(false);

    let when_show_text = move || is_active.get() || mobile.is_mobile.get();

    view! {
        <nav
            class=get_class
            on:mouseenter=move |_| is_active.set(true)
            on:mouseleave=move |_| is_active.set(false)
        >
            <div class="nav-bar__top-row">
                <A href="/dashboard" on:click=close_menu>
                    <div class="nav-bar__logo">
                        <LogoIcon />
                        <Show when=when_show_text>
                            <h5>GigLog</h5>
                        </Show>
                    </div>
                </A>
                <button
                    type="button"
                    class="nav-bar__menu-button"
                    aria-label="Toggle navigation menu"
                    on:click=toggle_mobile_menu
                >
                    <HamburgerIcon />
                </button>
            </div>
            <NavMenu items=items is_active=is_active is_mobile_menu_open=is_mobile_menu_open />
        </nav>
    }
}
