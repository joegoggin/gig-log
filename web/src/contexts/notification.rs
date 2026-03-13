use leptos::prelude::*;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    Success,
    Error,
    Info,
    Warning,
}

#[derive(Debug, Clone)]
pub struct NotificationInfo {
    pub id: u64,
    pub title: String,
    pub message: String,
    pub notification_type: NotificationType,
}

#[derive(Debug, Clone, Copy)]
pub struct NotificationContext {
    pub notifications: RwSignal<Vec<NotificationInfo>>,
    closing_ids: RwSignal<HashSet<u64>>,
    next_id: RwSignal<u64>,
}

impl NotificationContext {
    const CLOSE_ANIMATION_MS: u32 = 300;

    pub fn new() -> Self {
        Self {
            notifications: RwSignal::new(Vec::new()),
            closing_ids: RwSignal::new(HashSet::new()),
            next_id: RwSignal::new(0),
        }
    }

    pub fn show(&self, title: String, message: String, notification_type: NotificationType) {
        let id = self.next_id.get();
        self.next_id.set(id + 1);

        let notification = NotificationInfo {
            id,
            title,
            message,
            notification_type,
        };

        self.notifications.update(|n| n.push(notification));

        let notifications = *self;

        gloo_timers::callback::Timeout::new(5_000, move || {
            notifications.dismiss(id);
        })
        .forget();
    }

    pub fn show_success(&self, title: impl Into<String>, message: impl Into<String>) {
        self.show(title.into(), message.into(), NotificationType::Success);
    }

    pub fn show_error(&self, title: impl Into<String>, message: impl Into<String>) {
        self.show(title.into(), message.into(), NotificationType::Error);
    }

    pub fn show_info(&self, title: impl Into<String>, message: impl Into<String>) {
        self.show(title.into(), message.into(), NotificationType::Info);
    }

    pub fn show_warning(&self, title: impl Into<String>, message: impl Into<String>) {
        self.show(title.into(), message.into(), NotificationType::Warning);
    }

    pub fn dismiss(&self, id: u64) {
        let exists = self
            .notifications
            .with_untracked(|n| n.iter().any(|notification| notification.id == id));

        if !exists {
            return;
        }

        let is_already_closing = self.closing_ids.with_untracked(|ids| ids.contains(&id));

        if is_already_closing {
            return;
        }

        self.closing_ids.update(|ids| {
            ids.insert(id);
        });

        let notifications = self.notifications;
        let closing_ids = self.closing_ids;
        gloo_timers::callback::Timeout::new(Self::CLOSE_ANIMATION_MS, move || {
            notifications.update(|n| n.retain(|notification| notification.id != id));
            closing_ids.update(|ids| {
                ids.remove(&id);
            });
        })
        .forget();
    }

    pub fn is_closing(&self, id: u64) -> bool {
        self.closing_ids.with(|ids| ids.contains(&id))
    }
}

pub fn provide_notification_context() -> NotificationContext {
    let ctx = NotificationContext::new();
    provide_context(ctx.clone());
    ctx
}

pub fn use_notifications() -> NotificationContext {
    use_context::<NotificationContext>().expect(
        "NotificationContext not provided. Wrap your app with provide_notification_context()",
    )
}
