use leptos::prelude::*;

use crate::{
    components::{button::Button, Card},
    contexts::use_auth,
    utils::class_name::ClassNameUtil,
};

#[component]
pub fn HomePageCta(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("home-page-cta", class);

    let cta = class_name.get_root_class();

    // Context
    let auth = use_auth();

    // Variables
    let user = auth.user;

    view! {
        <Card class=cta>
            <h3>"Ready to simplify freelance admin?"</h3>
            <p>"Create your account and start logging work in under five minutes."</p>
            <Show when=move || user.get().is_none()>
                <Button href="/auth/sign-up">"Sign Up Now"</Button>
            </Show>
            <Show when=move || user.get().is_some()>
                <Button href="/dashboard">"Open Dashboard"</Button>
            </Show>
        </Card>
    }
}
