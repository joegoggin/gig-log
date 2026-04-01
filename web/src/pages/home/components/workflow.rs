use leptos::prelude::*;

use crate::{components::Card, utils::class_name::ClassNameUtil};

#[component]
pub fn HomePageWorkflow(#[prop(optional, into)] class: Option<String>) -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("home-page-workflow", class);

    let workflow = class_name.get_root_class();
    let list = class_name.get_sub_class("list");

    view! {
        <Card class=workflow>
            <h3>"Start in four simple steps"</h3>
            <ol class=list>
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
    }
}
