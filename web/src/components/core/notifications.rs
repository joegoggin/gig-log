use leptos::prelude::*;

use crate::{
    components::{CloseIcon, ErrorIcon, InfoIcon, SuccessIcon, WarningIcon},
    contexts::{NotificationType, use_notifications},
};

#[component]
pub fn Notifications() -> impl IntoView {
    let notifications = use_notifications();

    view! {
        <div class="notifications">
            <For each=move || notifications.notifications.get() key=|n| n.id let(notification)>
                {
                    let id = notification.id;
                    let notification_type = notification.notification_type.clone();
                    let class = move || {
                        let mut class = match &notification_type {
                            NotificationType::Success => "notification notification--success",
                            NotificationType::Error => "notification notification--error",
                            NotificationType::Info => "notification notification--info",
                            NotificationType::Warning => "notification notification--warning",
                        }
                            .to_string();
                        if notifications.is_closing(id) {
                            class.push_str(" notification--closing");
                        }
                        class
                    };

                    view! {
                        <div class=class>
                            <div class="notification__icon">
                                {match notification.notification_type {
                                    NotificationType::Success => SuccessIcon().into_any(),
                                    NotificationType::Error => ErrorIcon().into_any(),
                                    NotificationType::Info => InfoIcon().into_any(),
                                    NotificationType::Warning => WarningIcon().into_any(),
                                }}
                            </div>
                            <div class="notification__message">
                                <h5>{notification.title}</h5>
                                <p>{notification.message}</p>
                            </div>
                            <button
                                class="notification__close"
                                on:click=move |_| notifications.dismiss(id)
                            >
                                <CloseIcon />
                            </button>
                        </div>
                    }
                }
            </For>
        </div>
    }
}
