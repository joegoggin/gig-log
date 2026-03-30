//! CSS class string composition helpers.

/// Provides helper methods for composing CSS class name strings.
pub struct ClassNameUtil;

impl ClassNameUtil {
    /// Appends one class string to another.
    ///
    /// # Arguments
    ///
    /// * `current_class` — Existing class list.
    /// * `new_class` — Class string to append.
    ///
    /// # Returns
    ///
    /// A [`String`] containing both class segments.
    pub fn add_class(current_class: impl Into<String>, new_class: impl Into<String>) -> String {
        format!("{} {}", current_class.into(), new_class.into())
    }

    /// Appends an optional class string when present.
    ///
    /// # Arguments
    ///
    /// * `current_class` — Existing class list.
    /// * `optional_class` — Optional class string to append.
    ///
    /// # Returns
    ///
    /// A [`String`] containing the combined class list.
    pub fn add_optional_class(
        current_class: impl Into<String>,
        optional_class: Option<&str>,
    ) -> String {
        match optional_class {
            Some(class) => format!("{} {}", current_class.into(), class),
            None => current_class.into(),
        }
    }
}
