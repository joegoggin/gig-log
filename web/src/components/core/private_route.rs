use leptos::prelude::*;
use leptos_router::{components::Redirect, MatchNestedRoutes, NestedRoute, PossibleRouteMatch};

use crate::contexts::use_auth;

#[component(transparent)]
pub fn PrivateRoute<Segments, ViewFactory, View>(
    path: Segments,
    view: ViewFactory,
) -> impl MatchNestedRoutes + Clone
where
    Segments: PossibleRouteMatch + Clone + Send + 'static,
    ViewFactory: Fn() -> View + Send + Clone + 'static,
    View: IntoView + 'static,
{
    let auth = use_auth();

    let guarded_view = move || {
        if auth.loading.get() {
            view! {
                <div class="loading">
                    <div class="loading__spinner"></div>
                </div>
            }
            .into_any()
        } else if auth.user.get().is_some() {
            view().into_view().into_any()
        } else {
            view! { <Redirect path="/login" /> }.into_any()
        }
    };

    NestedRoute::new(path, guarded_view).into_maybe_erased()
}
