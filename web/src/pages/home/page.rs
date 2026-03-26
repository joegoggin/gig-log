use leptos::prelude::*;

use crate::{
    components::{Card, button::Button},
    contexts::use_auth,
    layouts::auth::AuthLayout,
    pages::home::components::{HomePageWorkflow, benefits::HomePageBenefits, hero::HomePageHero},
};

#[component]
pub fn HomePage() -> impl IntoView {
    let auth = use_auth();
    let user = auth.user;

    view! {
        <AuthLayout class="home-page">
            <HomePageHero />
            <HomePageBenefits />
            <HomePageWorkflow />
            <Card class="home-page__final-cta">
                <h3>"Ready to simplify freelance admin?"</h3>
                <p>"Create your account and start logging work in under five minutes."</p>
                <Show when=move || user.get().is_none()>
                    <Button href="/auth/sign-up">"Sign Up Now"</Button>
                </Show>
                <Show when=move || user.get().is_some()>
                    <Button href="/dashboard">"Open Dashboard"</Button>
                </Show>
            </Card>
        </AuthLayout>
    }
}
