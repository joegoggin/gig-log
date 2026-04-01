//! Page component for `ConfirmEmailPage`.

use leptos::prelude::*;

use crate::components::Card;
use crate::layouts::auth::AuthLayout;
use crate::utils::class_name::ClassNameUtil;

/// Renders the `ConfirmEmailPage` component.
///
/// # Returns
///
/// A Leptos view for the `ConfirmEmailPage` UI.
#[component]
pub fn ConfirmEmailPage() -> impl IntoView {
    // Classes
    let class_name = ClassNameUtil::new("confirm-email-page", None);
    let confirm_email_page = class_name.get_root_class();
    let card = class_name.get_sub_class("card");

    view! {
        <AuthLayout class=confirm_email_page>
            <Card class=card>
                <h1>"Confirm Email"</h1>
            </Card>
        </AuthLayout>
    }
}
