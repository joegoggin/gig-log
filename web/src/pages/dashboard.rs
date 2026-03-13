use leptos::prelude::*;

use crate::contexts::use_notifications;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let notifications = use_notifications();

    Effect::new(move || {
        untrack(|| {
            notifications.show_success("Success", "This is a success notification!");
            notifications.show_error("Error", "This is an error notification!");
            notifications.show_info("Info", "This is an info notification!");
            notifications.show_warning("Warning", "This is a warning notification!");
        });
    });

    view! {
        <div>
            <h1>"Dashboard"</h1>
        </div>
    }
}
