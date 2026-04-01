use leptos::prelude::*;

use crate::{
    components::{
        button::{Button, ButtonVariant},
        Card, LogoIcon,
    },
    contexts::use_auth,
    utils::class_name::ClassNameUtil,
};

#[component]
pub fn HomePageHero(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("home-page-hero", class);

    let home_page_hero = class_name.get_root_class();
    let logo = class_name.get_sub_class("logo");
    let buttons = class_name.get_sub_class("buttons");
    let grid = class_name.get_sub_class("grid");
    let grid_card = class_name.get_sub_class("grid-card");

    // Context
    let auth = use_auth();

    view! {
        <Card class=home_page_hero>
            <div class=logo>
                <LogoIcon />
                <div>
                    <p>"GigLog"</p>
                    <p>"Freelance work tracking, made practical"</p>
                </div>
            </div>
            <h1>"Track work. Stay paid. Keep every client organized."</h1>
            <p>
                "GigLog helps you capture sessions, monitor unpaid work, and maintain clear records without bloated admin tools."
            </p>
            <div class=buttons>
                <Show
                    when=move || !auth.is_authenticated()
                    fallback=move || view! { <Button href="/dashboard">"Go to Dashboard"</Button> }
                >
                    <Button href="/auth/sign-up">"Create Free Account"</Button>
                    <Button href="/auth/log-in" variant=ButtonVariant::Secondary>
                        "Log In"
                    </Button>
                </Show>
            </div>
            <div class=grid>
                <div class=grid_card.clone()>
                    <h2>"Fast daily logging"</h2>
                    <p>"Add jobs and sessions in minutes, right after each shift."</p>
                </div>
                <div class=grid_card.clone()>
                    <h2>"Payment clarity"</h2>
                    <p>"See what is paid, what is pending, and what needs follow-up."</p>
                </div>
                <div class=grid_card>
                    <h2>"Tax-ready history"</h2>
                    <p>"Keep clean records for reconciliation, reporting, and tax season."</p>
                </div>
            </div>
        </Card>
    }
}
