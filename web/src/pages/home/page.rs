use leptos::prelude::*;

use crate::{
    components::{
        Card, LogoIcon,
        button::{Button, ButtonVariant},
    },
    contexts::use_auth,
    layouts::auth::AuthLayout,
    pages::home::components::hero::HomePageHero,
};

#[component]
pub fn HomePage() -> impl IntoView {
    let auth = use_auth();
    let user = auth.user;

    view! {
        <AuthLayout class="home-page">
            <HomePageHero />
            <Card class="home-page__benefits">
                <h3>"Why freelancers choose GigLog"</h3>
                <ul class="home-page__benefit-list">
                    <li>
                        <span>"One timeline for every client"</span>
                        <p>
                            "Keep jobs, sessions, and payouts connected so context never gets lost."
                        </p>
                    </li>
                    <li>
                        <span>"Built for focus"</span>
                        <p>"Use a simple workflow designed around what you need every day."</p>
                    </li>
                    <li>
                        <span>"Follow up with confidence"</span>
                        <p>
                            "Spot unpaid work quickly and reach out before invoices slip through."
                        </p>
                    </li>
                    <li>
                        <span>"Reliable records when it counts"</span>
                        <p>
                            "Review completed work and earnings without piecing data together later."
                        </p>
                    </li>
                </ul>
            </Card>

            <Card class="home-page__workflow">
                <h3>"Start in four simple steps"</h3>
                <ol class="home-page__workflow-list">
                    <li>
                        <span>"1. Create a job"</span>
                        <p>"Set up the client, role, and base rate once."</p>
                    </li>
                    <li>
                        <span>"2. Log each session"</span>
                        <p>"Capture hours, dates, and notes while details are fresh."</p>
                    </li>
                    <li>
                        <span>"3. Track payment status"</span>
                        <p>"Mark invoices and payouts so outstanding work is always visible."</p>
                    </li>
                    <li>
                        <span>"4. Review your history"</span>
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
