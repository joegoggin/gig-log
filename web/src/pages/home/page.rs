use leptos::prelude::*;

use crate::{
    layouts::auth::AuthLayout,
    pages::home::components::{
        HomePageCta, HomePageWorkflow, benefits::HomePageBenefits, hero::HomePageHero,
    },
    utils::class_name::ClassNameUtil,
};

#[component]
pub fn HomePage() -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("home-page", None);

    let home_page = class_name.get_root_class();

    view! {
        <AuthLayout class=home_page>
            <HomePageHero />
            <HomePageBenefits />
            <HomePageWorkflow />
            <HomePageCta />
        </AuthLayout>
    }
}
