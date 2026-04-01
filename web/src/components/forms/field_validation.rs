//! Shared validation helpers for form field components.

use gig_log_common::models::error::ValidationError;
use leptos::prelude::*;

/// Stores reactive validation state for a single form field.
#[derive(Debug, Clone, Copy)]
pub struct FieldValidationState {
    /// Stores the current CSS class name for the field container.
    pub class_name: RwSignal<String>,
    /// Stores whether the field currently has a validation error.
    pub has_error: RwSignal<bool>,
    /// Stores the active validation error for this field.
    pub error: RwSignal<Option<ValidationError>>,
}

impl FieldValidationState {
    /// Creates field validation state synchronized with shared errors.
    ///
    /// # Arguments
    ///
    /// * `field` — Field name used to match validation errors.
    /// * `errors` — Signal containing all validation errors.
    /// * `normal_class` — CSS class used when no error exists.
    /// * `error_class` — CSS class used when an error exists.
    ///
    /// # Returns
    ///
    /// An initialized [`FieldValidationState`].
    pub fn new(
        field: impl Into<String>,
        errors: RwSignal<Vec<ValidationError>>,
        normal_class: String,
        error_class: String,
    ) -> Self {
        let field = field.into();

        let class_name = RwSignal::new(normal_class.clone());
        let has_error = RwSignal::new(false);
        let error = RwSignal::new(None);

        let class_name_signal = class_name;
        let has_error_signal = has_error;
        let error_signal = error;

        Effect::new(move || {
            let next_error = errors
                .get()
                .into_iter()
                .find(|validation_error| validation_error.field.as_deref() == Some(field.as_str()));

            if next_error.is_some() {
                class_name_signal.set(error_class.clone());
            } else {
                class_name_signal.set(normal_class.clone());
            }

            has_error_signal.set(next_error.is_some());
            error_signal.set(next_error);
        });

        Self {
            class_name,
            has_error,
            error,
        }
    }

    /// Clears this field's validation error from the shared error signal.
    ///
    /// # Arguments
    ///
    /// * `field` — Field name used to remove matching errors.
    /// * `errors` — Signal containing all validation errors.
    pub fn clear_error(&self, field: &str, errors: RwSignal<Vec<ValidationError>>) {
        if !self.has_error.get_untracked() {
            return;
        }

        let next_errors = errors.with_untracked(|current_errors| {
            current_errors
                .iter()
                .filter(|error| error.field.as_deref() != Some(field))
                .cloned()
                .collect::<Vec<_>>()
        });

        self.has_error.set(false);
        self.error.set(None);
        errors.set(next_errors);
    }
}
