use leptos::prelude::*;

use crate::{
    components::{Card, button::Button},
    contexts::use_auth,
    layouts::auth::AuthLayout,
    pages::home::components::{benefits::HomePageBenefits, hero::HomePageHero},
};

#[component]
pub fn HomePage() -> impl IntoView {
    let auth = use_auth();
    let user = auth.user;

    view! {
        <AuthLayout class="home-page">
            <HomePageHero />
            <HomePageBenefits />
            <Card class="home-page__workflow">
                <h3>"Start in four simple steps"</h3>
                <ol class="home-page__workflow-list">
                    <li>
                        <span>"Create a job"</span>
                        <p>"Set up the client, role, and base rate once."</p>
                    </li>
                    <li>
                        <span>"Log each session"</span>
                        <p>"Capture hours, dates, and notes while details are fresh."</p>
                    </li>
                    <li>
                        <span>"Track payment status"</span>
                        <p>"Mark invoices and payouts so outstanding work is always visible."</p>
                    </li>
                    <li>
                        <span>"Review your history"</span>
                        <p>"Use your records for monthly reviews and year-end reporting."</p>
                    </li>
                </ol>
            </Card>

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
