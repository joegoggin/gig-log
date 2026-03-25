use leptos::{attr::loading, prelude::*};
use leptos_router::{MatchNestedRoutes, NestedRoute, PossibleRouteMatch, components::Redirect};

use crate::{contexts::use_auth, utils::class_name::ClassNameUtil};

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
            view! { <Redirect path="/login" /> }.into_any()
        }
    };

    NestedRoute::new(path, guarded_view).into_maybe_erased()
}
