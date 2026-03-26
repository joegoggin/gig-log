use leptos::prelude::*;

use crate::{components::Card, utils::class_name::ClassNameUtil};

#[component]
pub fn HomePageBenefits() -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("home-page-benefits", None);

    let home_page_benefits = class_name.get_root_class();
    let list = class_name.get_sub_class("list");

    view! {
        <Card class=home_page_benefits>
            <h3>"Why freelancers choose GigLog"</h3>
            <ul class=list>
                <li>
                    <span>"One timeline for every client"</span>
                    <p>"Keep jobs, sessions, and payouts connected so context never gets lost."</p>
                </li>
                <li>
                    <span>"Built for focus"</span>
                    <p>"Use a simple workflow designed around what you need every day."</p>
                </li>
                <li>
                    <span>"Follow up with confidence"</span>
                    <p>"Spot unpaid work quickly and reach out before invoices slip through."</p>
                </li>
                <li>
                    <span>"Reliable records when it counts"</span>
                    <p>"Review completed work and earnings without piecing data together later."</p>
                </li>
            </ul>
        </Card>
    }
}
