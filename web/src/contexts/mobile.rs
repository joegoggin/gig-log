use leptos::prelude::*;
use std::cell::RefCell;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{MediaQueryList, MediaQueryListEvent, window};

const MOBILE_MEDIA_QUERY: &str = "(max-width: 1025px)";

thread_local! {
    static MOBILE_MEDIA_LISTENER: RefCell<Option<(MediaQueryList, Closure<dyn FnMut(MediaQueryListEvent)>)>> =
        RefCell::new(None);
}

fn get_media_query() -> Option<MediaQueryList> {
    window().and_then(|window| window.match_media(MOBILE_MEDIA_QUERY).ok().flatten())
}

fn clear_media_listener() {
    MOBILE_MEDIA_LISTENER.with(|listener| {
        if let Some((media_query, on_change)) = listener.borrow_mut().take() {
            let _ = media_query
                .remove_event_listener_with_callback("change", on_change.as_ref().unchecked_ref());
        }
    });
}

#[derive(Debug, Clone, Copy)]
pub struct MobileContext {
    pub is_mobile: RwSignal<bool>,
}

impl MobileContext {
    pub fn new(is_mobile: bool) -> Self {
        Self {
            is_mobile: RwSignal::new(is_mobile),
        }
    }
}

pub fn provide_mobile_context() -> MobileContext {
    let media_query = get_media_query();

    let ctx = MobileContext::new(media_query.as_ref().is_some_and(|query| query.matches()));
    provide_context(ctx);

    clear_media_listener();

    if let Some(media_query) = media_query {
        let is_mobile = ctx.is_mobile;
        let on_change =
            Closure::<dyn FnMut(MediaQueryListEvent)>::new(move |event: MediaQueryListEvent| {
                is_mobile.set(event.matches());
            });

        if media_query
            .add_event_listener_with_callback("change", on_change.as_ref().unchecked_ref())
            .is_ok()
        {
            MOBILE_MEDIA_LISTENER.with(|listener| {
                *listener.borrow_mut() = Some((media_query, on_change));
            });

            on_cleanup(clear_media_listener);
        }
    }

    ctx
}

pub fn use_mobile() -> MobileContext {
    use_context::<MobileContext>()
        .expect("MobileContext not provided. Wrap your app with provide_mobile_context()")
}
