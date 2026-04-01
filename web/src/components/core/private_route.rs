//! Route guard component for authenticated-only routes.

use leptos::prelude::*;
use leptos_router::{MatchNestedRoutes, NestedRoute, PossibleRouteMatch, components::Redirect};

use crate::{contexts::use_auth, utils::class_name::ClassNameUtil};

/// Creates a route that only renders when a user is authenticated.
///
/// When auth state is loading, this component renders a loading placeholder.
/// If no user is authenticated, it redirects to `/login`.
///
/// # Arguments
///
/// * `path` — Route path matcher for the guarded route.
/// * `view` — View factory used to render protected content.
///
/// # Returns
///
/// A nested route matcher that enforces authentication checks.
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
    // Context
    let auth = use_auth();

    let guarded_view = move || {
        if auth.loading.get() {
            // Classes
            let class_name = ClassNameUtil::new("loading", None);
            let loading = class_name.get_root_class();
            let spinner = class_name.get_sub_class("spinner");

            view! {
                <div class=loading>
                    <div class=spinner></div>
                </div>
            }
            .into_any()
        } else if auth.user.get().is_some() {
            view().into_view().into_any()
        } else {
            view! { <Redirect path="/auth/log-in" /> }.into_any()
        }
    };

    NestedRoute::new(path, guarded_view).into_maybe_erased()
}
