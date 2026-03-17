use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{
    CompanyIcon, HomeIcon, JobIcon, LogOutIcon, LogoIcon, PaymentIcon,
    nav::menu::{NavItem, NavMenu},
    settings::SettingsIcon,
};

#[component]
pub fn NavBar() -> impl IntoView {
    let is_active = RwSignal::new(false);

    let items: Vec<NavItem> = vec![
        NavItem::new("Dashboard", "/dashboard", HomeIcon.into()),
        NavItem::new("Companies", "/companies", CompanyIcon.into()),
        NavItem::new("Jobs", "/jobs", JobIcon.into()),
        NavItem::new("Payments", "/payments", PaymentIcon.into()),
        NavItem::new("Settings", "/settings", SettingsIcon.into()),
    ];

    view! {
        <nav
            class="nav-bar"
            on:mouseenter=move |_| is_active.set(true)
            on:mouseleave=move |_| is_active.set(false)
        >
            <A href="/dashboard">
                <div class="nav-bar__logo">
                    <LogoIcon />
                    <Show when=move || is_active.get()>
                        <h5>GigLog</h5>
                    </Show>
                </div>
            </A>
            <NavMenu items=items is_active=is_active />
        </nav>
    }
}
