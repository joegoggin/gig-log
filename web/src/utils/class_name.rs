//! CSS class string composition helpers.

/// Provides helper methods for composing CSS class name strings.
pub struct ClassNameUtil {
    main_class: String,
    class_prop: Option<String>,
    parent_class: Option<String>,
    is_layout: bool,
}

impl ClassNameUtil {
    /// Creates a utility for non-layout component class composition.
    ///
    /// # Arguments
    ///
    /// * `main_class` — Base root class name for the component.
    /// * `class_prop` — Optional additional class names from component props.
    ///
    /// # Returns
    ///
    /// An initialized [`ClassNameUtil`] for regular components.
    pub fn new(main_class: impl Into<String>, class_prop: Option<String>) -> Self {
        Self {
            main_class: main_class.into(),
            class_prop,
            parent_class: None,
            is_layout: false,
        }
    }

    /// Creates a utility for layout class composition.
    ///
    /// # Arguments
    ///
    /// * `main_class` — Base layout class name.
    /// * `class_prop` — Optional class names appended to the layout content element.
    ///
    /// # Returns
    ///
    /// An initialized [`ClassNameUtil`] configured for layout helpers.
    pub fn new_layout_class_name(
        main_class: impl Into<String>,
        class_prop: Option<String>,
    ) -> Self {
        Self {
            main_class: main_class.into(),
            class_prop,
            parent_class: None,
            is_layout: true,
        }
    }

    /// Creates a utility for components that combine parent and child root classes.
    ///
    /// # Arguments
    ///
    /// * `parent_class` — Shared parent class applied alongside the root class.
    /// * `main_class` — Base root class name for the component.
    /// * `class_prop` — Optional additional class names from component props.
    ///
    /// # Returns
    ///
    /// An initialized [`ClassNameUtil`] with a parent class context.
    pub fn new_with_parent(
        parent_class: impl Into<String>,
        main_class: impl Into<String>,
        class_prop: Option<String>,
    ) -> Self {
        Self {
            main_class: main_class.into(),
            class_prop,
            parent_class: Some(parent_class.into()),
            is_layout: false,
        }
    }

    /// Returns the root class, including optional component class props.
    ///
    /// # Returns
    ///
    /// A [`String`] containing the component root class.
    pub fn get_root_class(&self) -> String {
        match &self.class_prop {
            Some(class_prop) if !self.is_layout => format!("{} {}", self.main_class, class_prop),
            _ => self.main_class.to_string(),
        }
    }

    /// Returns the root class prefixed with the configured parent class.
    ///
    /// # Returns
    ///
    /// A [`String`] containing parent and root classes.
    pub fn get_root_class_with_parent(&self) -> String {
        match &self.parent_class {
            Some(parent_class) if !self.is_layout => {
                let root_class = self.get_root_class();

                format!("{} {}", parent_class, root_class)
            }
            _ => self.get_root_class(),
        }
    }

    /// Returns parent/root classes plus a parent variation class.
    ///
    /// # Arguments
    ///
    /// * `variation` — Modifier suffix appended to the parent class.
    ///
    /// # Returns
    ///
    /// A [`String`] containing parent, parent variation, and root classes.
    pub fn get_root_class_with_parent_variation(&self, variation: &str) -> String {
        match &self.parent_class {
            Some(parent_class) => {
                let root_class = self.get_root_class();
                let parent_variation = format!("{}--{}", parent_class, variation);

                format!("{} {} {}", parent_class, parent_variation, root_class)
            }
            None => self.get_root_class(),
        }
    }

    /// Returns a BEM-style sub-class for the configured root class.
    ///
    /// # Arguments
    ///
    /// * `sub_class` — Sub-element name appended as `__{sub_class}`.
    ///
    /// # Returns
    ///
    /// A [`String`] containing the generated sub-class name.
    pub fn get_sub_class(&self, sub_class: &str) -> String {
        format!("{}__{}", self.main_class, sub_class)
    }

    /// Returns the root class plus a root-level variation class.
    ///
    /// # Arguments
    ///
    /// * `variation` — Modifier suffix appended to the root class.
    ///
    /// # Returns
    ///
    /// A [`String`] containing root and variation classes.
    pub fn get_root_variation(&self, variation: &str) -> String {
        let root_class = self.get_root_class();
        let variation_class = format!("{}--{}", self.main_class, variation);

        format!("{} {}", root_class, variation_class)
    }

    /// Returns a sub-class plus its variation class.
    ///
    /// # Arguments
    ///
    /// * `sub_class` — Sub-element name for the generated sub-class.
    /// * `variation` — Modifier suffix appended to the sub-class.
    ///
    /// # Returns
    ///
    /// A [`String`] containing sub-class and sub-class variation names.
    pub fn get_sub_class_variation(&self, sub_class: &str, variation: &str) -> String {
        let sub_class = self.get_sub_class(sub_class);

        format!("{} {}--{}", sub_class, sub_class, variation)
    }

    /// Returns the layout content class, including layout class props when present.
    ///
    /// # Returns
    ///
    /// A [`String`] containing the layout content class value.
    pub fn get_content_class(&self) -> String {
        let content_class = format!("{}__content", self.main_class);

        if let Some(class_prop) = &self.class_prop
            && self.is_layout
        {
            return format!("{} {}", content_class, class_prop);
        }

        content_class
    }
}
