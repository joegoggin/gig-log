//! Notification context state and helper methods.

use leptos::prelude::*;
use std::collections::HashSet;

/// Represents supported notification style variants.
#[derive(Debug, Clone, PartialEq)]
pub enum NotificationType {
    /// Displays a success notification style.
    Success,
    /// Displays an error notification style.
    Error,
    /// Displays an informational notification style.
    Info,
    /// Displays a warning notification style.
    Warning,
}

/// Stores a single notification entry rendered in the UI.
#[derive(Debug, Clone)]
pub struct NotificationInfo {
    /// Stores the unique notification identifier.
    pub id: u64,
    /// Stores the title shown in the notification.
    pub title: String,
    /// Stores the body message shown in the notification.
    pub message: String,
    /// Stores the notification visual type.
    pub notification_type: NotificationType,
}

/// Stores notification queue state and lifecycle helpers.
#[derive(Debug, Clone, Copy)]
pub struct NotificationContext {
    /// Stores active notification entries.
    pub notifications: RwSignal<Vec<NotificationInfo>>,
    /// Stores IDs currently playing close animations.
    closing_ids: RwSignal<HashSet<u64>>,
    /// Stores the next incrementing notification ID.
    next_id: RwSignal<u64>,
}

impl NotificationContext {
    const CLOSE_ANIMATION_MS: u32 = 300;

    /// Creates a new empty [`NotificationContext`].
    ///
    /// # Returns
    ///
    /// An initialized [`NotificationContext`] with no notifications.
    pub fn new() -> Self {
        Self {
            notifications: RwSignal::new(Vec::new()),
            closing_ids: RwSignal::new(HashSet::new()),
            next_id: RwSignal::new(0),
        }
    }

    /// Shows a notification and schedules automatic dismissal.
    ///
    /// # Arguments
    ///
    /// * `title` — Notification title text.
    /// * `message` — Notification message text.
    /// * `notification_type` — Notification style variant.
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

    /// Shows a success notification.
    ///
    /// # Arguments
    ///
    /// * `title` — Notification title text.
    /// * `message` — Notification message text.
    pub fn show_success(&self, title: impl Into<String>, message: impl Into<String>) {
        self.show(title.into(), message.into(), NotificationType::Success);
    }

    /// Shows an error notification.
    ///
    /// # Arguments
    ///
    /// * `title` — Notification title text.
    /// * `message` — Notification message text.
    pub fn show_error(&self, title: impl Into<String>, message: impl Into<String>) {
        self.show(title.into(), message.into(), NotificationType::Error);
    }

    /// Shows an informational notification.
    ///
    /// # Arguments
    ///
    /// * `title` — Notification title text.
    /// * `message` — Notification message text.
    pub fn show_info(&self, title: impl Into<String>, message: impl Into<String>) {
        self.show(title.into(), message.into(), NotificationType::Info);
    }

    /// Shows a warning notification.
    ///
    /// # Arguments
    ///
    /// * `title` — Notification title text.
    /// * `message` — Notification message text.
    pub fn show_warning(&self, title: impl Into<String>, message: impl Into<String>) {
        self.show(title.into(), message.into(), NotificationType::Warning);
    }

    /// Starts dismissal for a notification by ID.
    ///
    /// # Arguments
    ///
    /// * `id` — Identifier of the notification to dismiss.
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

    /// Returns whether a notification is currently closing.
    ///
    /// # Arguments
    ///
    /// * `id` — Identifier of the notification.
    ///
    /// # Returns
    ///
    /// A [`bool`] indicating whether close animation is active.
    pub fn is_closing(&self, id: u64) -> bool {
        self.closing_ids.with(|ids| ids.contains(&id))
    }
}

/// Provides the shared notification context.
///
/// # Returns
///
/// The created [`NotificationContext`] inserted into Leptos context.
pub fn provide_notification_context() -> NotificationContext {
    let ctx = NotificationContext::new();
    provide_context(ctx.clone());
    ctx
}

/// Retrieves the shared notification context.
///
/// # Returns
///
/// The current [`NotificationContext`] from Leptos context.
pub fn use_notifications() -> NotificationContext {
    use_context::<NotificationContext>().expect(
        "NotificationContext not provided. Wrap your app with provide_notification_context()",
    )
}
