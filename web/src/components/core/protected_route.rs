use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::contexts::use_auth;

#[component]
pub fn ProtectedRoute(children: ChildrenFn) -> impl IntoView {
    let auth = use_auth();
    let navigate = use_navigate();

    view! {
        {move || {
            let loading = auth.loading.get();
            let user = auth.user.get();
            if loading {

                view! {
                    <div class="loading">
                        <div class="loading__spinner"></div>
                    </div>
                }
                    .into_any()
            } else if user.is_some() {
                children().into_any()
            } else {
                navigate("/login", Default::default());
                view! {
                    <div class="loading">
                        <div class="loading__spinner"></div>
                    </div>
                }
                    .into_any()
            }
        }}
    }
}
